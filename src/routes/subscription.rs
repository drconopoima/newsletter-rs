use crate::subscription::{FormData, SubscriptionFormData};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use deadpool_postgres::{Object, Pool};
use std::convert::TryFrom;
use std::sync::Arc;
use tokio_postgres::Statement;
use uuid::{NoContext, Timestamp, Uuid};

#[tracing::instrument(
    name = "Processing incoming subscription.",
    skip( form, request ),
    fields(
        subscription_email = %form.email,
        subscription_name = %form.name
    )
)]
pub async fn subscription(request: HttpRequest, form: web::Form<FormData>) -> impl Responder {
    let subscription_form: SubscriptionFormData = match SubscriptionFormData::try_from(form.0) {
        Ok(form_data) => form_data,
        Err(error) => {
            tracing::error!("routes/subscription.rs {}", error);
            return HttpResponse::BadRequest().body(error);
        }
    };
    let optional_postgres_pool: Option<&Arc<Pool>> = match request.app_data::<Arc<Pool>>() {
        Some(postgres_pool) => Some(postgres_pool),
        None => {
            tracing::error!("Could not retrieve postgres pool from app_data.");
            None
        }
    };
    if optional_postgres_pool.is_none() {
        return HttpResponse::InternalServerError()
            .body("DB pool error while processing subscription.");
    }
    let postgres_pool = optional_postgres_pool.unwrap();
    let optional_postgres_client = get_postgres_client(postgres_pool).await;
    if optional_postgres_client.is_none() {
        return HttpResponse::InternalServerError()
            .body("DB client error while processing subscription.");
    }
    let postgres_client = optional_postgres_client.unwrap();
    run_insert_subscriber_query(postgres_client, subscription_form).await
}

#[tracing::instrument(name = "Retrieving database client from pool.", skip(postgres_pool))]
pub async fn get_postgres_client(postgres_pool: &Arc<Pool>) -> Option<Object> {
    match postgres_pool.get().await {
        Ok(manager) => Some(manager),
        Err(error) => {
            tracing::error!("Could not retrieve postgres client from pool, {}.", error);
            None
        }
    }
}

#[tracing::instrument(
    name = "Preparing cached insert subscription query statement.",
    skip(postgres_client)
)]
pub async fn prepare_cached_statement(postgres_client: &Object) -> Option<Statement> {
    match postgres_client
        .prepare_cached(
            r#"
                INSERT INTO newsletter.subscription (id, email, name)
                VALUES ($1, $2, $3)
            "#,
        )
        .await
    {
        Ok(statement) => Some(statement),
        Err(error) => {
            tracing::error!(
                "Failed to prepare cached insert subscription query: {}",
                error
            );
            None
        }
    }
}

#[tracing::instrument(
    "Running insert query to save subscription into database.",
    skip(postgres_client, form)
)]
pub async fn run_insert_subscriber_query(
    postgres_client: Object,
    form: SubscriptionFormData,
) -> HttpResponse {
    let statement = prepare_cached_statement(&postgres_client).await;
    if statement.is_none() {
        return HttpResponse::InternalServerError().finish();
    }
    let generated_uuid: Uuid = Uuid::new_v7(Timestamp::now(NoContext));
    match postgres_client
        .query(
            &statement.unwrap(),
            &[&generated_uuid, &form.email.as_ref(), &form.name.as_ref()],
        )
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(error) => {
            tracing::warn!("Failed to insert subscription: {}", error);
            let error_message = error.to_string();
            if error_message
                .starts_with("db error: ERROR: duplicate key value violates unique constraint")
            {
                return HttpResponse::BadRequest().body(format!(
                    "Input error, email '{}' is already subscribed.",
                    &form.email
                ));
            }
            HttpResponse::InternalServerError().body("DB error while inserting subscription")
        }
    }
}

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use deadpool_postgres::Pool;
use regex::Regex;
use std::sync::Arc;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct SubscriptionFormData {
    email: String,
    name: String,
}

pub async fn subscription(
    request: HttpRequest,
    form: web::Form<SubscriptionFormData>,
) -> impl Responder {
    let generated_uuid: Uuid = Uuid::new_v4();
    let postgres_pool: &Arc<Pool> = request.app_data::<Arc<Pool>>().unwrap();
    let postgres_client = match postgres_pool.get().await {
        Ok(manager) => Some(manager),
        Err(error) => {
            println!(
                "Failed to retrieve postgres connection from pool: {}",
                error
            );
            None
        }
    };
    if postgres_client.is_none() {
        HttpResponse::InternalServerError().finish();
    }
    let postgres_client = postgres_client.unwrap();
    let email_format = Regex::new(r"^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap();
    if email_format.is_match(&form.email) {
        let statement = match postgres_client
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
                println!(
                    "Failed to prepare cached insert subscription query: {}",
                    error
                );
                None
            }
        };
        if statement.is_none() {
            HttpResponse::InternalServerError().finish();
        }
        match postgres_client
            .query(
                &statement.unwrap(),
                &[&generated_uuid, &form.email, &form.name],
            )
            .await
        {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(error) => {
                println!("Failed to insert subscription: {}", error);
                HttpResponse::InternalServerError().finish()
            }
        }
    } else {
        HttpResponse::BadRequest().finish()
    }
}

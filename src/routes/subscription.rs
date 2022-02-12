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
    let postgres_client = postgres_pool.get().await.unwrap();
    let email_format = Regex::new(r"^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap();
    if email_format.is_match(&form.email) {
        let statement = postgres_client
            .prepare_cached(
                r#"
                    INSERT INTO newsletter.subscription (id, email, name)
                    VALUES ($1, $2, $3)
                "#,
            )
            .await
            .expect("Failed to prepare insert query.");
        postgres_client
            .query(&statement, &[&generated_uuid, &form.email, &form.name])
            .await
            .expect("Failed to insert requested subscription.");
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::BadRequest().finish()
    }
}

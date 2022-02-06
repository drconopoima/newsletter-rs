use crate::postgres::NoTlsPostgresConnection;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::sync::Arc;
use uuid::Uuid;
use regex::Regex;
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
    let connection: &Arc<NoTlsPostgresConnection> =
        request.app_data::<Arc<NoTlsPostgresConnection>>().unwrap();
    let email_format = Regex::new(r"^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap();
    if email_format.is_match(&form.email) {
        connection
        .client
        .query(
            r#"
                    INSERT INTO newsletter.subscription (id, email, name)
                    VALUES ($1, $2, $3)
                "#,
            &[&generated_uuid, &form.email, &form.name],
        )
        .await
        .expect("Failed to insert requested subscription.");
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::BadRequest().finish()
    }    
}

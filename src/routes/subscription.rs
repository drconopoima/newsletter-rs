use crate::postgres::NoTlsPostgresConnection;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
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
    let generated_uuid: String = format!("{}", Uuid::new_v4());
    println!("email: {}, name: {}", form.email, form.name);
    let connection: &Arc<NoTlsPostgresConnection> =
        request.app_data::<Arc<NoTlsPostgresConnection>>().unwrap();
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
}

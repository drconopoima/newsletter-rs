use actix_web::{web, HttpResponse};

#[derive(serde::Deserialize)]
pub struct SubscriptionFormData {
    pub email: String,
    pub name: String,
}

pub async fn subscription(_form: web::Form<SubscriptionFormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

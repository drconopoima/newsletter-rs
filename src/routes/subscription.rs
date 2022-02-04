use actix_web::{HttpResponse, web::{self, Data} };
use tokio_postgres::Client;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct SubscriptionFormData {
    email: String,
    name: String,
}

pub async fn subscription(client: Data<Client>,form: web::Form<SubscriptionFormData>) -> HttpResponse {
    let generated_uuid: String = format!("{}",Uuid::new_v4());
    println!("email: {}, name: {}", form.email, form.name);
    let client: &Client = client.get_ref();
    println!("{:?}",client);
    client
        .query(
        r#"
                        INSERT INTO subscriptions (id, email, name)
                        VALUES ($1, $2, $3)
                    "#,
            &[&generated_uuid,&form.email, &form.name],
        )
        .await
        .expect("Failed to insert requested subscription.");
    HttpResponse::Ok().finish()
}

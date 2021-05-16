use newsletter_rs::run;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let bind_address: (&str, u16) = ("127.0.0.1", 8080);
    // Raises if failed to bind address
    let listener = TcpListener::bind(bind_address)?;
    // Run server on TcpListener
    run(listener)?.await
}

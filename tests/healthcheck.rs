use std::net::TcpListener;
#[actix_rt::test]
async fn healthcheck_endpoint() {
    // Arrange
    let server_route = launch_http_server();
    // Act
    // Client library makes HTTP requests against server
    let client = reqwest::Client::new();
    let healthcheck_route = &format!("{}/healthcheck", server_route);
    let response = client
        .get(healthcheck_route)
        .send()
        .await
        .expect(&format!("Failed GET request to {}", healthcheck_route));
    // Assert
    // Status 200 OK
    assert!(response.status().is_success());
    // Empty Body
    assert_eq!(Some(0), response.content_length());
}

// Launch an instance for our HTTP server in the background
fn launch_http_server() -> String {
    let local_addr = "127.0.0.1";
    let address: (&str, u16) = (local_addr, 0);
    let listener = TcpListener::bind(address).expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = newsletter_rs::run(listener).expect("Failed to listen on address");
    let _ = tokio::spawn(server);
    format!("http://{}:{}", local_addr, port)
}

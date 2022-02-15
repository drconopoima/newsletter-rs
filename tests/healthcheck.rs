use deadpool_postgres::Pool;
use newsletter_rs::{
    configuration::{get_configuration, MigrationSettings},
    postgres::migrate_database,
};
use std::net::TcpListener;
use uuid::Uuid;

pub struct ServerPostgres {
    pub address: String,
    pub postgres_pool: Pool,
}

// Launch an instance for our HTTP server in the background
async fn launch_http_server() -> ServerPostgres {
    let config_file: &str = "configuration";
    let mut configuration = get_configuration(config_file).unwrap_or_else(|error| {
        panic!(
            "ERROR: Failed to read configuration file '{}': {}",
            &config_file, error
        )
    });
    let migration_settings = MigrationSettings {
        migrate: true,
        folder: Some("./migrations".to_string()),
    };
    configuration.database_migration = Some(migration_settings);
    let isolated_database_name = Uuid::new_v4().to_string();
    let uuid_without_hyphens = isolated_database_name.replace("-", "");
    configuration.database.database = Some(uuid_without_hyphens.to_string());
    let postgres_pool: Pool = migrate_database(
        configuration.database,
        configuration
            .database_migration
            .as_ref()
            .unwrap()
            .folder
            .as_ref()
            .unwrap()
            .to_string(),
    )
    .await;
    let local_addr = "127.0.0.1";
    let address: (&str, u16) = (local_addr, 0);
    let listener = TcpListener::bind(address).expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = newsletter_rs::startup::run(listener, postgres_pool.clone())
        .expect("Failed to listen on address");
    let _ = tokio::spawn(server);
    ServerPostgres {
        address: format!("http://{}:{}", local_addr, port),
        postgres_pool,
    }
}

#[derive(serde::Serialize)]
struct Body {
    email: String,
    name: String,
}

#[tokio::test]
async fn healthcheck_endpoint() {
    // Arrange
    let server_postgres = launch_http_server().await;
    let client = reqwest::Client::new();
    // Act
    // Client library makes HTTP requests against server
    let healthcheck_route = &format!("{}/healthcheck", server_postgres.address);
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

#[tokio::test]
async fn subscription_200_valid_form_data() {
    // Arrange
    let server_postgres = launch_http_server().await;
    let client = reqwest::Client::new();
    let email_field = "email_nobody_has@drconopoima.com";
    let name_field = "Jane Doe";
    let body = Body {
        email: email_field.to_string(),
        name: name_field.to_string(),
    };
    let body_encoded = serde_urlencoded::to_string(&body).unwrap();
    let subscriptions_route = &format!("{}/subscription", server_postgres.address);
    // Act
    let response = client
        .post(subscriptions_route)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body_encoded)
        .send()
        .await
        .expect(&format!("Failed POST request to {}", subscriptions_route));
    // Assert
    assert_eq!(200, response.status().as_u16());
    // Act

    let client = server_postgres
        .postgres_pool
        .get()
        .await
        .expect("Failed to generate client connection to postgres from pool");
    let row_results = client
        .query(
            "SELECT email,name FROM newsletter.subscription WHERE email=$1::TEXT",
            &[&email_field],
        )
        .await
        .expect("Failed to fetch saved subscription.");
    // Assert
    let retrieved_email: &str = row_results[0].get(&"email");
    let retrieved_name: &str = row_results[0].get(&"name");
    assert_eq!(&retrieved_email, &email_field);
    assert_eq!(&retrieved_name, &name_field);
}

#[tokio::test]
async fn subscription_400_incomplete_form_data() {
    // Arrange
    let server_postgres = launch_http_server().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=Jane%20Doe", "missing email"),
        ("email=email_nobody_has%40drconopoima.com", "missing name"),
        ("", "missing email and name"),
    ];
    let subscriptions_route = &format!("{}/subscription", server_postgres.address);
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(subscriptions_route)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect(&format!("Failed POST request to {}", subscriptions_route));
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Custom message for particular test case failure
            "Expected API failure response code to be 400 Bad Request when body payload was {}.",
            error_message
        )
    }
}

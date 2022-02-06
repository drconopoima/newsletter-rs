use tokio::task::{spawn, JoinError};
use tokio_postgres::{connect, Client, NoTls};
/// Postgres interface
pub struct NoTlsPostgresConnection {
    pub client: Client,
    pub postgres_connection_string: String,
}

pub async fn connect_postgres(
    postgres_connection_string: String,
) -> Result<NoTlsPostgresConnection, JoinError> {
    let (client, connection) = connect(&postgres_connection_string, NoTls)
        .await
        .unwrap_or_else(|error| {
            panic!(
                "ERROR: Failed to connect to Postgres at URL '{}', {}",
                &postgres_connection_string, error
            )
        });
    // Spawn connection
    let join_handle = spawn(async move {
        if let Err(error) = connection.await {
            panic!("Connection error with postgres {}", error);
        }
    });
    join_handle.await?;
    Ok(NoTlsPostgresConnection {
        client,
        postgres_connection_string,
    })
}

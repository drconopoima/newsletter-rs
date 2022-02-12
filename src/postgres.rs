use deadpool_postgres::{Manager, ManagerConfig, Object, Pool, RecyclingMethod};
use std::str::FromStr;
use tokio_postgres::NoTls;

pub fn generate_connection_pool(postgres_connection_string: String) -> Pool {
    let postgres_configuration =
        tokio_postgres::Config::from_str(&postgres_connection_string).unwrap();
    let deadpool_manager_config = ManagerConfig {
        recycling_method: RecyclingMethod::Verified,
    };
    let deadpool_manager =
        Manager::from_config(postgres_configuration, NoTls, deadpool_manager_config);
    Pool::builder(deadpool_manager)
        .max_size(16)
        .build()
        .unwrap()
}

pub async fn get_client(pool: Pool) -> Object {
    pool.get().await.unwrap()
}

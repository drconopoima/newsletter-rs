use crate::routes::healthcheck_structs::HealthcheckObject;
use crate::routes::{healthcheck, subscription};
use actix_web::middleware::Logger;
use actix_web::{dev::Server, web, App, HttpServer};
use deadpool_postgres::Pool;
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use time::OffsetDateTime;

pub struct HealthcheckCache {
    pub valid_until: OffsetDateTime,
    pub healthcheck: HealthcheckObject,
}
pub struct CachedHealthcheck {
    pub cache: Option<HealthcheckCache>,
    pub validity_period: Duration,
}

pub fn run(
    listener: TcpListener,
    postgres_pool: Pool,
    admin_bind_address: Option<(String, u16)>,
    healthcheck_validity_period_ms: Option<Duration>,
) -> Result<(Server, Option<Server>), std::io::Error> {
    let postgres_pool = Arc::new(postgres_pool);
    let cached_healthcheck: CachedHealthcheck;
    let healthcheck_validity_period: Duration;
    if let Some(healthcheck_validity) = healthcheck_validity_period_ms {
        healthcheck_validity_period = healthcheck_validity;
    } else {
        healthcheck_validity_period = Duration::from_millis(1000);
    }
    cached_healthcheck = CachedHealthcheck {
        cache: None,
        validity_period: healthcheck_validity_period,
    };
    let arc_cached_healthcheck: Arc<RwLock<CachedHealthcheck>> =
        Arc::new(RwLock::from(cached_healthcheck));
    if admin_bind_address.is_none() {
        let server = HttpServer::new(move || {
            App::new()
                // Logging middleware
                .wrap(Logger::default())
                // Ensure App to be running correctly
                .route("/healthcheck", web::get().to(healthcheck))
                // Handle newsletter subscription requests
                .route("/subscription", web::post().to(subscription))
                // Register the Postgres connection as part of application state
                .app_data(postgres_pool.clone())
                // Register cache for healthcheck endpoint
                .app_data(arc_cached_healthcheck.clone())
        })
        .listen(listener)?
        .run();
        return Ok((server, None));
    }
    let admin_bind_address = admin_bind_address.unwrap();
    let admin_listener = TcpListener::bind(admin_bind_address)?;
    let postgres_pool1 = postgres_pool.clone();
    let arc_cached_healthcheck1 = arc_cached_healthcheck.clone();
    let server1 = HttpServer::new(move || {
        App::new()
            // Logging middleware
            .wrap(Logger::default())
            // Handle newsletter subscription requests
            .route("/subscription", web::post().to(subscription))
            // Register the Postgres connection as part of application state
            .app_data(postgres_pool1.clone())
            // Register cache for healthcheck endpoint
            .app_data(arc_cached_healthcheck1.clone())
    })
    .listen(listener)?
    .run();
    let server2 = HttpServer::new(move || {
        App::new()
            // Logging middleware
            .wrap(Logger::default())
            // Ensure App to be running correctly
            .route("/healthcheck", web::get().to(healthcheck))
            // Register the Postgres connection as part of application state
            .app_data(postgres_pool.clone())
            // Register cache for healthcheck endpoint
            .app_data(arc_cached_healthcheck.clone())
    })
    .listen(admin_listener)?
    .run();
    Ok((server1, Some(server2)))
}

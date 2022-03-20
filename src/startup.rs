use crate::readiness::{probe_readiness, CachedHealth};
use crate::routes::{healthcheck, subscription};
use actix_web::middleware::Logger;
use actix_web::{dev::Server, web, App, HttpServer};
use anyhow::{Context, Result};
use deadpool_postgres::Pool;
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;

pub fn run(
    listener: TcpListener,
    postgres_pool: Pool,
    admin_bind_address: Option<(String, u16)>,
    healthcheck_validity_period_ms: Option<Duration>,
) -> Result<(Server, Option<Server>)> {
    let postgres_pool = Arc::new(postgres_pool);
    let healthcheck_validity_period: Duration =
        if let Some(healthcheck_validity) = healthcheck_validity_period_ms {
            healthcheck_validity
        } else {
            Duration::from_millis(1000)
        };
    let cached_healthcheck = CachedHealth { cache: None };
    let arc_cached_healthcheck: Arc<RwLock<CachedHealth>> =
        Arc::new(RwLock::from(cached_healthcheck));
    if admin_bind_address.is_none() {
        let server = HttpServer::new(move || {
            let arc_cached_healthcheck_readiness = arc_cached_healthcheck.clone();
            let postgres_pool_readiness = postgres_pool.clone();
            tokio::task::spawn_blocking(move || {
                let mut interval = tokio::time::interval(healthcheck_validity_period);
                loop {
                    futures::executor::block_on(interval.tick());
                    let healthresponse = futures::executor::block_on(probe_readiness(
                        postgres_pool_readiness.clone(),
                    ));
                    if let Ok(mut cache) = arc_cached_healthcheck_readiness.write() {
                        cache.cache = Some(healthresponse);
                    }
                }
            });
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
    let admin_listener = TcpListener::bind(&admin_bind_address).with_context(|| {
        format!(
            "{}::startup::run: Failed to open a TCP Listener on address '{}' and port '{}'.",
            env!("CARGO_PKG_NAME"),
            admin_bind_address.0,
            admin_bind_address.1
        )
    })?;
    let postgres_pool1 = postgres_pool.clone();
    let server1 = HttpServer::new(move || {
        App::new()
            // Logging middleware
            .wrap(Logger::default())
            // Handle newsletter subscription requests
            .route("/subscription", web::post().to(subscription))
            // Register the Postgres connection as part of application state
            .app_data(postgres_pool1.clone())
    })
    .listen(listener)?
    .run();
    let server2 = HttpServer::new(move || {
        let arc_cached_healthcheck_readiness = arc_cached_healthcheck.clone();
        let postgres_pool_readiness = postgres_pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut interval = tokio::time::interval(healthcheck_validity_period);
            loop {
                futures::executor::block_on(interval.tick());
                let healthresponse = futures::executor::block_on(probe_readiness(
                    postgres_pool_readiness.clone(),
                ));
                if let Ok(mut cache) = arc_cached_healthcheck_readiness.write() {
                    cache.cache = Some(healthresponse);
                }
            }
        });
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

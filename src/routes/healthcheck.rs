use crate::readiness::CachedHealthcheck;
use actix_web::{HttpRequest, HttpResponse, Responder};
use std::sync::{Arc, RwLock};

pub async fn healthcheck(request: HttpRequest) -> impl Responder {
    let optional_cache_rwlock: Option<&Arc<RwLock<CachedHealthcheck>>> =
        match request.app_data::<Arc<RwLock<CachedHealthcheck>>>() {
            Some(cache_rwlock) => Some(cache_rwlock),
            None => {
                tracing::error!("Could not retrieve cached healthcheck from app_data.");
                None
            }
        };
    if optional_cache_rwlock.is_some() {
        let cache_rwlock = optional_cache_rwlock.unwrap();
        if let Ok(cache) = cache_rwlock.try_read() {
            if let Some(healthcheck) = &cache.cache {
                return HttpResponse::Ok().json(&healthcheck);
            }
        }
    };
    HttpResponse::InternalServerError().finish()
}

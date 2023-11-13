use crate::readiness::{
    build_postgres_readwrite_response, to_rfc3339, CachedHealth, STATUS_FAIL, STATUS_WARN,
};
use actix_web::{HttpRequest, HttpResponse, Responder};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

pub async fn healthcheck(request: HttpRequest) -> impl Responder {
    let optional_cache_rwlock: Option<&Arc<RwLock<CachedHealth>>> =
        match request.app_data::<Arc<RwLock<CachedHealth>>>() {
            Some(cache_rwlock) => Some(cache_rwlock),
            None => {
                tracing::error!("Could not retrieve cached healthcheck from app_data.");
                None
            }
        };
    if let Some(cache_rwlock) = optional_cache_rwlock {
        if let Ok(cache) = cache_rwlock.try_read() {
            if let Some(healthcheck) = &cache.cache {
                return HttpResponse::Ok().json(healthcheck);
            }
        }
    };
    let now_systemtime = SystemTime::now();
    let now_string = to_rfc3339(now_systemtime).unwrap();
    HttpResponse::Ok().json(build_postgres_readwrite_response(
        STATUS_FAIL,
        STATUS_FAIL,
        STATUS_WARN,
        &now_string,
        "Could not read state.",
    ))
}

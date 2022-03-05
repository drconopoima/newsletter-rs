use actix_web::{HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use std::sync::Arc;
use std::time::SystemTime;

// Healthcheck response format for HTTP APIs https://inadarei.github.io/rfc-healthcheck/
#[derive(serde::Serialize)]
pub struct Healthcheck {
    pub status: String,
    pub checks: ChecksObject,
    pub output: String,
}

#[derive(serde::Serialize)]
pub struct ChecksObject {
    pub newsletter: NewsletterChecks,
    pub postgres_read: PostgresReadChecks,
    pub postgres_write: PostgresWriteChecks,
}

#[derive(serde::Serialize)]

pub struct PostgresReadChecks {
    pub status: String,
    pub time: String,
    pub output: String,
}

#[derive(serde::Serialize)]

pub struct PostgresWriteChecks {
    pub status: String,
    pub time: String,
    pub pg_isinrecovery: String,
    pub output: String,
}

#[derive(serde::Serialize)]

pub struct NewsletterChecks {
    pub status: String,
    pub time: String,
    pub output: String,
}

pub async fn healthcheck(request: HttpRequest) -> impl Responder {
    let optional_postgres_pool: Option<&Arc<Pool>> = match request.app_data::<Arc<Pool>>() {
        Some(postgres_pool) => Some(postgres_pool),
        None => {
            tracing::error!("Could not retrieve postgres pool from app_data.");
            None
        }
    };
    if optional_postgres_pool.is_none() {
        return HttpResponse::InternalServerError()
            .body("DB pool error while processing subscription.");
    }
    let postgres_pool = optional_postgres_pool.unwrap();
    let optional_postgres_client = match postgres_pool.get().await {
        Ok(manager) => Some(manager),
        Err(error) => {
            tracing::error!("Could not retrieve postgres client from pool, {}.", error);
            None
        }
    };
    if optional_postgres_client.is_none() {
        return HttpResponse::InternalServerError()
            .body("DB client error while processing subscription.");
    }
    let _postgres_client = optional_postgres_client.unwrap();

    let now_systemtime = SystemTime::now();
    let now_datetime: DateTime<Utc> = now_systemtime.into();
    let now_string = now_datetime.to_rfc3339();
    let status = "pass";
    let output = "";
    let postgres_read = PostgresReadChecks {
        status: status.to_owned(),
        time: now_string.to_owned(),
        output: output.to_owned(),
    };
    let postgres_write = PostgresWriteChecks {
        status: status.to_owned(),
        time: now_string.to_owned(),
        pg_isinrecovery: "false".to_owned(),
        output: output.to_owned(),
    };
    let newsletter = NewsletterChecks {
        status: status.to_owned(),
        time: now_string,
        output: output.to_owned(),
    };

    let checks = ChecksObject {
        newsletter,
        postgres_read,
        postgres_write,
    };

    let healthcheck = Healthcheck {
        status: status.to_owned(),
        checks,
        output: output.to_owned(),
    };

    HttpResponse::Ok().json(healthcheck)
}

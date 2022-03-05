use actix_web::{HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use std::sync::Arc;
use std::time::SystemTime;

// Healthcheck response format for HTTP APIs https://inadarei.github.io/rfc-healthcheck/
#[derive(serde::Serialize)]
pub struct HealthcheckObject {
    pub status: String,
    pub checks: ChecksObject,
    pub output: String,
    pub time: String,
}

#[derive(serde::Serialize)]
pub struct ChecksObject {
    pub postgres_read: PostgresReadChecks,
    pub postgres_write: PostgresWriteChecks,
}

#[derive(serde::Serialize)]

pub struct PostgresReadChecks {
    pub status: String,
    pub time: Option<String>,
    pub output: String,
}

#[derive(serde::Serialize)]

pub struct PostgresWriteChecks {
    pub status: String,
    pub time: Option<String>,
    pub pg_isinrecovery: Option<String>,
    pub output: String,
}

fn postgres_read_checks(status: &str, time: Option<String>, output: &str) -> PostgresReadChecks {
    return PostgresReadChecks {
        status: status.to_owned(),
        time,
        output: output.to_owned(),
    };
}

fn postgres_write_checks(
    status: &str,
    time: Option<String>,
    pg_isinrecovery: Option<String>,
    output: &str,
) -> PostgresWriteChecks {
    return PostgresWriteChecks {
        status: status.to_owned(),
        time,
        pg_isinrecovery,
        output: output.to_owned(),
    };
}

fn get_healthcheck_object(
    status: &str,
    time: &str,
    output: &str,
    postgres_read: PostgresReadChecks,
    postgres_write: PostgresWriteChecks,
) -> HealthcheckObject {
    let checks = ChecksObject {
        postgres_read,
        postgres_write,
    };

    HealthcheckObject {
        status: status.to_owned(),
        checks,
        time: time.to_owned(),
        output: output.to_owned(),
    }
}

pub async fn healthcheck(request: HttpRequest) -> impl Responder {
    let now_systemtime = SystemTime::now();
    let now_datetime: DateTime<Utc> = now_systemtime.into();
    let now_string = now_datetime.to_rfc3339();
    let status_pass = "pass";
    let status_fail = "fail";
    let status_warn = "warn";
    let output_pass = "";
    let global_status: &str;
    let postgres_read: PostgresReadChecks;
    let postgres_write: PostgresWriteChecks;
    let postgres_read_status: &str;
    let postgres_write_status: &str;
    let postgres_pool_error = "DB pool error.";
    let optional_postgres_pool: Option<&Arc<Pool>> = match request.app_data::<Arc<Pool>>() {
        Some(postgres_pool) => Some(postgres_pool),
        None => {
            tracing::error!("Could not retrieve postgres pool from app_data.");
            None
        }
    };
    if optional_postgres_pool.is_none() {
        postgres_read_status = status_fail;
        postgres_write_status = status_fail;
        global_status = status_warn;
        postgres_read = postgres_read_checks(postgres_read_status, None, postgres_pool_error);
        postgres_write =
            postgres_write_checks(postgres_write_status, None, None, postgres_pool_error);
        return HttpResponse::Ok().json(get_healthcheck_object(
            global_status,
            &now_string,
            output_pass,
            postgres_read,
            postgres_write,
        ));
    }
    let postgres_pool = optional_postgres_pool.unwrap();
    let postgres_client_error = "DB client error";
    let optional_postgres_client = match postgres_pool.get().await {
        Ok(manager) => Some(manager),
        Err(error) => {
            tracing::error!("Could not retrieve postgres client from pool, {}.", error);
            None
        }
    };
    if optional_postgres_client.is_none() {
        postgres_read_status = status_fail;
        postgres_write_status = status_fail;
        global_status = status_warn;
        postgres_read = postgres_read_checks(postgres_read_status, None, postgres_client_error);
        postgres_write =
            postgres_write_checks(postgres_write_status, None, None, postgres_client_error);
        return HttpResponse::Ok().json(get_healthcheck_object(
            global_status,
            &now_string,
            output_pass,
            postgres_read,
            postgres_write,
        ));
    }
    let _postgres_client = optional_postgres_client.unwrap();
    postgres_read_status = status_pass;
    postgres_write_status = status_pass;
    global_status = status_pass;
    postgres_read = postgres_read_checks(postgres_read_status, None, "");
    postgres_write = postgres_write_checks(postgres_write_status, None, None, "");

    HttpResponse::Ok().json(get_healthcheck_object(
        global_status,
        &now_string,
        output_pass,
        postgres_read,
        postgres_write,
    ))
}

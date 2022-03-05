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

fn postgres_read_write_fail_healthcheck(
    status_fail: &str,
    status_warn: &str,
    now_string: &str,
    output: &str,
) -> HealthcheckObject {
    let postgres_read_status = status_fail;
    let postgres_write_status = status_fail;
    let global_status = status_warn;
    let postgres_read = postgres_read_checks(postgres_read_status, None, output);
    let postgres_write = postgres_write_checks(postgres_write_status, None, None, output);
    get_healthcheck_object(global_status, now_string, "", postgres_read, postgres_write)
}

pub async fn healthcheck(request: HttpRequest) -> impl Responder {
    let now_systemtime = SystemTime::now();
    let now_datetime: DateTime<Utc> = now_systemtime.into();
    let now_string = now_datetime.to_rfc3339();
    let status_pass = "pass";
    let status_fail = "fail";
    let status_warn = "warn";
    let output_pass = "";
    let postgres_pool_error = "DB pool error.";
    let optional_postgres_pool: Option<&Arc<Pool>> = match request.app_data::<Arc<Pool>>() {
        Some(postgres_pool) => Some(postgres_pool),
        None => {
            tracing::error!("Could not retrieve postgres pool from app_data.");
            None
        }
    };
    if optional_postgres_pool.is_none() {
        return HttpResponse::Ok().json(postgres_read_write_fail_healthcheck(
            status_fail,
            status_warn,
            &now_string,
            postgres_pool_error,
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
        return HttpResponse::Ok().json(postgres_read_write_fail_healthcheck(
            status_fail,
            status_warn,
            &now_string,
            postgres_client_error,
        ));
    }
    let postgres_client = optional_postgres_client.unwrap();
    let statement_error = "DB statement error.";
    let statement_read = match postgres_client
        .prepare_cached(
            r#"
                SELECT clock_timestamp(),pg_is_in_recovery()
            "#,
        )
        .await
    {
        Ok(statement) => Some(statement),
        Err(error) => {
            tracing::error!("Failed to prepare cached healthcheck query: {}", error);
            None
        }
    };
    if statement_read.is_none() {
        return HttpResponse::Ok().json(postgres_read_write_fail_healthcheck(
            status_fail,
            status_warn,
            &now_string,
            statement_error,
        ));
    }

    let postgres_read = postgres_read_checks(status_pass, None, "");
    let postgres_write = postgres_write_checks(status_pass, None, None, "");

    HttpResponse::Ok().json(get_healthcheck_object(
        status_pass,
        &now_string,
        output_pass,
        postgres_read,
        postgres_write,
    ))
}

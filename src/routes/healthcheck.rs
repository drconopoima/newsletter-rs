use actix_web::{HttpRequest, HttpResponse, Responder};
use deadpool_postgres::Pool;
use std::sync::Arc;
use std::time::SystemTime;
use time::{error, format_description::well_known::Rfc3339, OffsetDateTime};

fn to_rfc3339<T>(datetime: T) -> Result<String, error::Format>
where
    T: Into<OffsetDateTime>,
{
    datetime.into().format(&Rfc3339)
}

// Healthcheck response format for HTTP APIs https://inadarei.github.io/rfc-healthcheck/
#[derive(serde::Serialize)]
pub struct HealthcheckObject {
    pub status: String,
    pub checks: ChecksObject,
    pub output: String,
    pub time: String,
    pub version: String,
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
    pub pg_is_in_recovery: Option<bool>,
    pub output: String,
}

fn postgres_read_checks(status: &str, time: Option<String>, output: &str) -> PostgresReadChecks {
    PostgresReadChecks {
        status: status.to_owned(),
        time,
        output: output.to_owned(),
    }
}

fn postgres_write_checks(
    status: &str,
    time: Option<String>,
    pg_is_in_recovery: Option<bool>,
    output: &str,
) -> PostgresWriteChecks {
    PostgresWriteChecks {
        status: status.to_owned(),
        time,
        pg_is_in_recovery,
        output: output.to_owned(),
    }
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
        version: env!("CARGO_PKG_VERSION").to_owned(),
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
    let now_string = to_rfc3339(now_systemtime).unwrap();
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
    let statement_read_error = "DB read statement error.";
    let statement_read = match postgres_client
        .prepare_cached(
            r#"
                SELECT clock_timestamp() as datetime,pg_is_in_recovery() as recovery
            "#,
        )
        .await
    {
        Ok(statement) => Some(statement),
        Err(error) => {
            tracing::error!("Failed to prepare cached healthcheck read query: {}", error);
            None
        }
    };
    if statement_read.is_none() {
        return HttpResponse::Ok().json(postgres_read_write_fail_healthcheck(
            status_fail,
            status_warn,
            &now_string,
            statement_read_error,
        ));
    }
    let read_error = "DB read error.";
    let optional_row = match postgres_client.query(&statement_read.unwrap(), &[]).await {
        Ok(row) => Some(row),
        Err(error) => {
            tracing::warn!("Failed healthcheck query: {}", error);
            None
        }
    };
    if optional_row.is_none() {
        return HttpResponse::Ok().json(postgres_read_write_fail_healthcheck(
            status_fail,
            status_warn,
            &now_string,
            read_error,
        ));
    }
    let row_results = optional_row.unwrap();
    let postgres_read_timestamp: OffsetDateTime = row_results[0].get(&"datetime");
    let postgres_read_timestamp_string = to_rfc3339(postgres_read_timestamp).unwrap();
    let postgres_recovery: bool = row_results[0].get(&"recovery");
    let postgres_read = postgres_read_checks(
        status_pass,
        Some(postgres_read_timestamp_string.to_owned()),
        output_pass,
    );
    let statement_write_error = "DB write statement error.";
    let statement_write = match postgres_client
        .prepare_cached(
            r#"
                UPDATE _healthcheck set updated_by=$1
                WHERE id=true RETURNING datetime
            "#,
        )
        .await
    {
        Ok(statement) => Some(statement),
        Err(error) => {
            tracing::error!(
                "Failed to prepare cached healthcheck write query: {}",
                error
            );
            None
        }
    };
    let postgres_write: PostgresWriteChecks;
    if statement_write.is_none() {
        postgres_write = postgres_write_checks(
            status_fail,
            None,
            Some(postgres_recovery),
            statement_write_error,
        );

        return HttpResponse::Ok().json(get_healthcheck_object(
            status_warn,
            &now_string,
            output_pass,
            postgres_read,
            postgres_write,
        ));
    }
    let updated_by_parameter = format!("newsletter-rs {}", &now_string);
    let write_error = "DB write error.";
    let optional_row = match postgres_client
        .query(&statement_write.unwrap(), &[&updated_by_parameter])
        .await
    {
        Ok(row) => Some(row),
        Err(error) => {
            tracing::warn!("Failed healthcheck query: {}", error);
            None
        }
    };
    if optional_row.is_none() {
        postgres_write =
            postgres_write_checks(status_fail, None, Some(postgres_recovery), write_error);

        return HttpResponse::Ok().json(get_healthcheck_object(
            status_warn,
            &now_string,
            output_pass,
            postgres_read,
            postgres_write,
        ));
    }
    let row_results = optional_row.unwrap();
    let postgres_write_timestamp: OffsetDateTime = row_results[0].get(&"datetime");
    let postgres_write_timestamp_string = to_rfc3339(postgres_write_timestamp).unwrap();
    postgres_write = postgres_write_checks(
        status_pass,
        Some(postgres_write_timestamp_string),
        Some(postgres_recovery),
        output_pass,
    );

    HttpResponse::Ok().json(get_healthcheck_object(
        status_pass,
        &now_string,
        output_pass,
        postgres_read,
        postgres_write,
    ))
}

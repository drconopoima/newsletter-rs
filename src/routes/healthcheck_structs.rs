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
    pub version: Option<String>,
}

#[derive(serde::Serialize)]

pub struct PostgresWriteChecks {
    pub status: String,
    pub time: Option<String>,
    pub pg_is_in_recovery: Option<bool>,
    pub output: String,
    pub version: Option<String>,
}

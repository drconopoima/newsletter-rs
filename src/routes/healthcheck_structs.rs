// Healthcheck response format for HTTP APIs https://inadarei.github.io/rfc-healthcheck/
#[derive(serde::Serialize, Clone)]
pub struct HealthcheckObject {
    pub status: String,
    pub checks: ChecksObject,
    pub output: String,
    pub time: String,
    pub version: String,
}

#[derive(serde::Serialize, Clone)]
pub struct ChecksObject {
    pub postgres_read: PostgresReadChecks,
    pub postgres_write: PostgresWriteChecks,
}

#[derive(serde::Serialize, Clone)]

pub struct PostgresReadChecks {
    pub status: String,
    pub time: Option<String>,
    pub output: String,
}

#[derive(serde::Serialize, Clone)]

pub struct PostgresWriteChecks {
    pub status: String,
    pub time: Option<String>,
    pub pg_is_in_recovery: Option<bool>,
    pub output: String,
}

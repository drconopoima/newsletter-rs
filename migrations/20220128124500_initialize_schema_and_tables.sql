BEGIN;

CREATE SCHEMA IF NOT EXISTS newsletter;

-- For initialization migrations
CREATE TABLE IF NOT EXISTS _initialization_migrations(
    version SERIAL PRIMARY KEY,
    filename TEXT NOT NULL,
    installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),
    md5_hash UUID NOT NULL
);

-- For eventual SQLx-handled migrations
CREATE TABLE IF NOT EXISTS _sqlx_migrations (
    version BIGINT PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),
    success BOOLEAN NOT NULL,
    checksum BYTEA NOT NULL,
    execution_time BIGINT NOT NULL
);

COMMIT;

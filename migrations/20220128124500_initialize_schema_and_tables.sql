BEGIN;

CREATE SCHEMA IF NOT EXISTS newsletter;

-- For initialization migrations
CREATE TABLE IF NOT EXISTS _initialization_migrations(
    version SERIAL PRIMARY KEY,
    filename TEXT UNIQUE NOT NULL,
    installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),
    md5_hash UUID UNIQUE NOT NULL
);

COMMIT;


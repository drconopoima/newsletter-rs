BEGIN;

CREATE UNLOGGED TABLE IF NOT EXISTS _healthcheck (
    id bool UNIQUE NOT NULL DEFAULT TRUE,
    datetime timestamptz NOT NULL DEFAULT NOW(),
    updated_by char varying(126) NOT NULL,
    CONSTRAINT _healthcheck_unique_row CHECK (id)
);

REVOKE DELETE, TRUNCATE ON public._healthcheck FROM public;

INSERT INTO _healthcheck (id, updated_by)
VALUES (true, '20220305131000_postgresql_healthcheck_table.sql')
ON CONFLICT ON CONSTRAINT _healthcheck_id_key 
DO 
    UPDATE SET updated_by = '20220305131000_postgresql_healthcheck_table.sql';

-- Create user 'newsletter'
DO
$do$
BEGIN
    IF NOT EXISTS (
        SELECT FROM pg_catalog.pg_roles
        WHERE  rolname = 'newsletter') THEN
        CREATE USER newsletter with ENCRYPTED PASSWORD 'password';
    END IF;
END
$do$;

GRANT ALL PRIVILEGES ON SCHEMA newsletter to newsletter;

ALTER TABLE newsletter.subscription
OWNER TO newsletter;

CREATE OR REPLACE FUNCTION trigger_set_timestamp()
RETURNS TRIGGER AS $_healthcheck_datetime$
    BEGIN
        NEW.datetime = NOW();
        RETURN NEW;
    END;
$_healthcheck_datetime$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER set_timestamp
BEFORE INSERT OR UPDATE ON _healthcheck
    FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();
REVOKE INSERT, UPDATE
ON TABLE "_healthcheck"
FROM public
cascade;

GRANT
    INSERT (id, updated_by),
    UPDATE (id, updated_by),
    SELECT (id, datetime, updated_by)
ON TABLE "_healthcheck"
TO public;

COMMIT;

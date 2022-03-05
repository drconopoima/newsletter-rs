BEGIN;

CREATE UNLOGGED TABLE IF NOT EXISTS _healthcheck (
   id bool PRIMARY KEY DEFAULT TRUE,
   datetime timestamptz DEFAULT NOW(),
   CONSTRAINT _healthcheck_unique_row CHECK (id)
);


CREATE OR REPLACE FUNCTION _healthcheck_ensure_timestamp_nonfuture() RETURNS TRIGGER AS $_healthcheck_timestamp_nonfuture$
    BEGIN
        --
        -- Raise error if a future datetime tries to be inserted
        --
        IF( NEW.datetime > sysdate ) THEN
            RAISE EXCEPTION 'Update of datetime is not allowed for dates in the future: %',
                NEW.datetime;
        END IF;
    END;
$_healthcheck_timestamp_nonfuture$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER _healthcheck_timestamp_nonfuture
BEFORE INSERT OR UPDATE ON _healthcheck
    FOR EACH ROW EXECUTE PROCEDURE _healthcheck_ensure_timestamp_nonfuture();

COMMIT;

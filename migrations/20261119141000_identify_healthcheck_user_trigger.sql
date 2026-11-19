BEGIN;

CREATE OR REPLACE FUNCTION trigger_healthcheck_audit()
RETURNS TRIGGER AS $$
BEGIN
    NEW.datetime := NOW();

    NEW.updated_by :=
        FORMAT(
            '[%s]%s',
            session_user,
            REGEXP_REPLACE(
                COALESCE(NEW.updated_by, ''),
                '^\[[^]]+\]',
                ''
            )
        );

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS set_timestamp ON _healthcheck;

CREATE TRIGGER set_timestamp
BEFORE INSERT OR UPDATE ON _healthcheck
FOR EACH ROW
EXECUTE FUNCTION trigger_healthcheck_audit();

COMMIT;

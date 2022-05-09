BEGIN;

CREATE PROCEDURE alters() AS $_$
BEGIN

IF NOT EXISTS (
    SELECT 1
    FROM INFORMATION_SCHEMA.COLUMNS
    WHERE table_schema = 'newsletter'
        AND table_name = 'subscription'
        AND column_name = 'name'
        AND is_nullable = 'NO'
        AND data_type = 'character varying'
        AND character_maximum_length = 126
)
THEN
    ALTER TABLE newsletter.subscription RENAME COLUMN name TO first_name;
    ALTER TABLE newsletter.subscription ALTER COLUMN first_name TYPE char varying(126);
    ALTER TABLE newsletter.subscription ALTER COLUMN first_name SET NOT NULL;
END IF;

IF NOT EXISTS (
    SELECT 1
    FROM INFORMATION_SCHEMA.COLUMNS
    WHERE table_schema = 'newsletter'
        AND table_name = 'subscription'
        AND column_name = 'last_name'
)
THEN
    ALTER TABLE newsletter.subscription ADD last_name char varying(126);
END IF;

IF NOT EXISTS (
    SELECT 1
    FROM INFORMATION_SCHEMA.COLUMNS
    WHERE table_name = '_initialization_migrations'
        AND column_name = 'filename'
        AND is_nullable = 'NO'
        AND data_type = 'character varying'
        AND character_maximum_length = 255
)
THEN
    ALTER TABLE _initialization_migrations ALTER COLUMN filename TYPE char varying(255);
    ALTER TABLE _initialization_migrations ALTER COLUMN filename SET NOT NULL;
END IF;
END; $_$ LANGUAGE plpgsql;

CALL alters();

COMMIT;

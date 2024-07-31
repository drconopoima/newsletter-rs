BEGIN;

-- Limit name field length to VARCHAR(254) trimming multiple whitespace where necessary
ALTER TABLE newsletter.subscription
  ALTER COLUMN name
    TYPE VARCHAR(254)
    USING LEFT( REGEXP_REPLACE( TRIM(name), '^\s+|\s+$|\s+', ' ','g' ) , 254 );

-- Sanitize name input on insert or update going forward
CREATE OR REPLACE FUNCTION trigger_name_trim_whitespace()
RETURNS TRIGGER AS $_subscription_name_trim_whitespace$
    BEGIN
        NEW.name = LEFT( REGEXP_REPLACE( TRIM(NEW.name), '^\s+|\s+$|\s+', ' ','g' ) , 254 );
        RETURN NEW;
    END;
$_subscription_name_trim_whitespace$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER name_trim_whitespace
BEFORE INSERT OR UPDATE ON newsletter.subscription
    FOR EACH ROW EXECUTE PROCEDURE trigger_name_trim_whitespace();

COMMIT;

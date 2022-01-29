BEGIN;
-- Enable case-insensitive text extension
CREATE EXTENSION IF NOT EXISTS citext;
-- Add constraint to email format
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'email') THEN
        CREATE DOMAIN email AS citext
            CHECK ( value ~ '^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$' );
    END IF;
END$$;
-- Create Subscription Table
CREATE TABLE IF NOT EXISTS newsletter.subscription(
    id uuid NOT NULL,
    PRIMARY KEY (id),
    email email NOT NULL UNIQUE,
    name TEXT NOT NULL,
    subscription_date timestamptz NOT NULL
);
COMMIT;

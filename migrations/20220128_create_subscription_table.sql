BEGIN;
-- Enable case-insensitive text extension
CREATE EXTENSION citext;
-- Add constraint to email format
CREATE DOMAIN email AS citext
    CHECK ( value ~ '^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$' );
-- Create Subscription Table
CREATE TABLE IF NOT EXISTS newsletter.subscription(
    id uuid NOT NULL,
    PRIMARY KEY (id),
    email email NOT NULL UNIQUE,
    name TEXT NOT NULL,
    subscribtion_date timestamptz NOT NULL
);
COMMIT;

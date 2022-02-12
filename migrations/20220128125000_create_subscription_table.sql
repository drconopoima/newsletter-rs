BEGIN;
-- Enable case-insensitive text extension
CREATE EXTENSION IF NOT EXISTS citext;

-- Create Subscription Table
CREATE TABLE IF NOT EXISTS newsletter.subscription(
    id uuid NOT NULL,
    PRIMARY KEY (id),
    email citext NOT NULL UNIQUE,
    name TEXT NOT NULL,
    subscription_date timestamptz NOT NULL DEFAULT now()
);
COMMIT;

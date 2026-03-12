-- Add migration script here
-- Add subscriber status
ALTER TABLE subscriptions
ADD COLUMN status TEXT NOT NULL DEFAULT 'pending_confirmation';

-- Table for confirmation tokens
CREATE TABLE subscription_tokens (
    subscription_token TEXT PRIMARY KEY,
    subscriber_id UUID NOT NULL REFERENCES subscriptions(id)
);

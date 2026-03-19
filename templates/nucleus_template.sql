-- ============================================================
-- Nucleus Federated Mode — Database Template
-- ============================================================
--
-- This template creates tables in YOUR database to store user
-- data synced from Nucleus via webhooks.
--
-- Nucleus stores authentication data (credentials, sessions, MFA).
-- Your database stores user profile data + your custom data.
--
-- Sync mechanism: Nucleus sends webhook events (user.created,
-- user.updated, user.deleted) to your webhook endpoint.
-- Your backend processes these events and updates these tables.
--
-- Usage:
--   psql -d your_database -f nucleus_template.sql
--
-- After applying this template, you can add your own columns
-- to nucleus_users or create related tables.
-- ============================================================

-- Users synced from Nucleus
CREATE TABLE IF NOT EXISTS nucleus_users (
    -- Nucleus user ID (matches the 'sub' claim in JWT)
    id UUID PRIMARY KEY,

    -- Profile data (synced from Nucleus webhooks)
    email VARCHAR(255) NOT NULL,
    email_verified BOOLEAN NOT NULL DEFAULT false,
    phone VARCHAR(50),
    phone_verified BOOLEAN NOT NULL DEFAULT false,
    username VARCHAR(100),
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    avatar_url TEXT,

    -- Custom metadata from Nucleus (public metadata only)
    metadata JSONB NOT NULL DEFAULT '{}',

    -- Sync tracking
    nucleus_synced_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Standard timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_nucleus_users_email
    ON nucleus_users(email);
CREATE INDEX IF NOT EXISTS idx_nucleus_users_username
    ON nucleus_users(username) WHERE username IS NOT NULL;

-- Organizations synced from Nucleus
CREATE TABLE IF NOT EXISTS nucleus_organizations (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    logo_url TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    nucleus_synced_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_nucleus_orgs_slug
    ON nucleus_organizations(slug);

-- Organization memberships synced from Nucleus
CREATE TABLE IF NOT EXISTS nucleus_org_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES nucleus_organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES nucleus_users(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    permissions TEXT[] DEFAULT '{}',
    nucleus_synced_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(org_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_nucleus_members_user
    ON nucleus_org_members(user_id);
CREATE INDEX IF NOT EXISTS idx_nucleus_members_org
    ON nucleus_org_members(org_id);

-- ============================================================
-- Webhook processing helper
-- ============================================================

-- Track processed webhook events (idempotency)
CREATE TABLE IF NOT EXISTS nucleus_webhook_log (
    event_id UUID PRIMARY KEY,
    event_type VARCHAR(100) NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Function to check if a webhook event was already processed
-- Usage: SELECT nucleus_is_event_processed('event-uuid-here');
CREATE OR REPLACE FUNCTION nucleus_is_event_processed(p_event_id UUID)
RETURNS BOOLEAN AS $$
BEGIN
    RETURN EXISTS (SELECT 1 FROM nucleus_webhook_log WHERE event_id = p_event_id);
END;
$$ LANGUAGE plpgsql;

-- Function to mark a webhook event as processed
CREATE OR REPLACE FUNCTION nucleus_mark_event_processed(p_event_id UUID, p_event_type VARCHAR)
RETURNS VOID AS $$
BEGIN
    INSERT INTO nucleus_webhook_log (event_id, event_type)
    VALUES (p_event_id, p_event_type)
    ON CONFLICT (event_id) DO NOTHING;
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- Example: Adding your own columns
-- ============================================================
--
-- ALTER TABLE nucleus_users ADD COLUMN subscription_tier VARCHAR(20);
-- ALTER TABLE nucleus_users ADD COLUMN preferences JSONB DEFAULT '{}';
-- ALTER TABLE nucleus_users ADD COLUMN internal_id SERIAL;
--
-- CREATE TABLE orders (
--     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
--     user_id UUID NOT NULL REFERENCES nucleus_users(id),
--     ...
-- );
-- ============================================================

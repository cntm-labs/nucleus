-- Seed a system account + system project with id = Uuid::nil() so that
-- bootstrap operations referencing the "system" scope (currently: signing key
-- generation in main.rs:66, where default_project_id = Uuid::nil()) can satisfy
-- the foreign key constraints in signing_keys, jwt_templates, etc.
--
-- This row is internal-only — its email is non-functional, password_hash is a
-- non-verifiable sentinel (anti-enumeration handles the "no real user found"
-- path), and the project is hidden from dashboard listings by id filter on the
-- caller side. The row exists solely to anchor system-owned rows.
--
-- Idempotent: ON CONFLICT DO NOTHING so re-running a migration is safe.

INSERT INTO accounts (id, email, password_hash, name, company, is_active, email_verified)
VALUES (
    '00000000-0000-0000-0000-000000000000',
    'system@nucleus.internal',
    '!!system-account-no-login!!',
    'System',
    'Nucleus',
    false,
    true
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO projects (id, account_id, name, slug, data_mode, environment, plan_id, is_active)
SELECT
    '00000000-0000-0000-0000-000000000000',
    '00000000-0000-0000-0000-000000000000',
    'System',
    'system',
    'centralized',
    'production',
    plans.id,
    false
FROM plans
WHERE plans.name = 'free'
ON CONFLICT (id) DO NOTHING;

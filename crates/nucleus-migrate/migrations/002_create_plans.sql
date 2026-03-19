CREATE TABLE plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL,
    max_users INT,
    max_orgs INT,
    max_mau INT,
    max_api_requests BIGINT,
    features JSONB NOT NULL DEFAULT '{}',
    price_monthly DECIMAL(10,2),
    price_yearly DECIMAL(10,2),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Seed default plans
INSERT INTO plans (name, display_name, max_users, max_orgs, max_mau, max_api_requests, features, price_monthly, price_yearly) VALUES
('free', 'Free', 100, 3, 1000, 10000, '{"mfa": true, "saml": false, "custom_domain": false, "audit_log": false}', 0, 0),
('pro', 'Pro', 10000, 50, 10000, 1000000, '{"mfa": true, "saml": false, "custom_domain": true, "audit_log": true}', 25, 250),
('enterprise', 'Enterprise', NULL, NULL, NULL, NULL, '{"mfa": true, "saml": true, "custom_domain": true, "audit_log": true}', NULL, NULL);

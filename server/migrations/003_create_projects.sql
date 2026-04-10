CREATE TYPE data_mode AS ENUM ('centralized', 'federated');
CREATE TYPE environment_type AS ENUM ('development', 'staging', 'production');

CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES accounts(id),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    data_mode data_mode NOT NULL DEFAULT 'centralized',
    environment environment_type NOT NULL DEFAULT 'development',
    plan_id UUID NOT NULL REFERENCES plans(id),
    webhook_url TEXT,
    webhook_secret TEXT,
    allowed_origins TEXT[] DEFAULT '{}',
    session_ttl INT NOT NULL DEFAULT 604800,
    jwt_lifetime INT NOT NULL DEFAULT 300,
    jwt_algorithm VARCHAR(10) NOT NULL DEFAULT 'RS256',
    settings JSONB NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_projects_account_id ON projects(account_id);
CREATE INDEX idx_projects_slug ON projects(slug);

CREATE TABLE subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id),
    plan_id UUID NOT NULL REFERENCES plans(id),
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    cancel_at TIMESTAMPTZ,
    external_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE usage_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id),
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    mau_count INT NOT NULL DEFAULT 0,
    api_request_count BIGINT NOT NULL DEFAULT 0,
    auth_event_count BIGINT NOT NULL DEFAULT 0,
    webhook_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(project_id, period_start)
);

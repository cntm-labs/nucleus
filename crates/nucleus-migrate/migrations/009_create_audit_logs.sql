CREATE TABLE sign_in_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id),
    user_id UUID REFERENCES users(id),
    method VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL,
    failure_reason VARCHAR(100),
    ip INET,
    user_agent TEXT,
    country_code VARCHAR(2),
    city VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_sign_in_attempts_project_created ON sign_in_attempts(project_id, created_at);
CREATE INDEX idx_sign_in_attempts_user_created ON sign_in_attempts(user_id, created_at);

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id),
    actor_type VARCHAR(20) NOT NULL,
    actor_id UUID,
    action VARCHAR(100) NOT NULL,
    target_type VARCHAR(50),
    target_id UUID,
    metadata JSONB NOT NULL DEFAULT '{}',
    ip INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_audit_logs_project_created ON audit_logs(project_id, created_at);
CREATE INDEX idx_audit_logs_project_action ON audit_logs(project_id, action);

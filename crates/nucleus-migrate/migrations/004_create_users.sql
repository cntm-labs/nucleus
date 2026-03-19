CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id),
    external_id VARCHAR(255),
    email VARCHAR(255) NOT NULL,
    email_verified BOOLEAN NOT NULL DEFAULT false,
    phone VARCHAR(50),
    phone_verified BOOLEAN NOT NULL DEFAULT false,
    username VARCHAR(100),
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    avatar_url TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    private_metadata JSONB NOT NULL DEFAULT '{}',
    last_sign_in_at TIMESTAMPTZ,
    banned_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(project_id, email)
);

CREATE UNIQUE INDEX idx_users_project_username ON users(project_id, username) WHERE username IS NOT NULL;
CREATE INDEX idx_users_project_external_id ON users(project_id, external_id) WHERE external_id IS NOT NULL;
CREATE INDEX idx_users_project_id ON users(project_id);
CREATE INDEX idx_users_deleted_at ON users(deleted_at) WHERE deleted_at IS NOT NULL;

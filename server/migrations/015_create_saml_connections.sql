CREATE TABLE saml_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id),
    name VARCHAR(255) NOT NULL,
    entity_id VARCHAR(512) NOT NULL,
    sso_url TEXT NOT NULL,
    certificate TEXT NOT NULL,
    attribute_mapping JSONB NOT NULL DEFAULT '{"email": "email", "first_name": "firstName", "last_name": "lastName"}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(project_id, entity_id)
);

CREATE INDEX idx_saml_connections_project ON saml_connections(project_id);

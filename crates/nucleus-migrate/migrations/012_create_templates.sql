CREATE TABLE notification_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id),
    template_type VARCHAR(10) NOT NULL,
    event VARCHAR(100) NOT NULL,
    subject TEXT,
    body_html TEXT,
    body_text TEXT,
    is_custom BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(project_id, template_type, event)
);

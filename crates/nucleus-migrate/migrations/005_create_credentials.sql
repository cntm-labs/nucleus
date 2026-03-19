CREATE TYPE credential_type AS ENUM ('password', 'oauth', 'magic_link', 'passkey', 'otp');

CREATE TABLE credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    credential_type credential_type NOT NULL,
    identifier TEXT,
    secret_hash TEXT,
    provider VARCHAR(50),
    provider_data JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX idx_credentials_type_provider_identifier
    ON credentials(credential_type, provider, identifier)
    WHERE identifier IS NOT NULL;
CREATE INDEX idx_credentials_user_id ON credentials(user_id);

CREATE TABLE user_security (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    failed_attempts INT NOT NULL DEFAULT 0,
    locked_until TIMESTAMPTZ,
    lockout_count INT NOT NULL DEFAULT 0,
    password_changed_at TIMESTAMPTZ,
    force_password_change BOOLEAN NOT NULL DEFAULT false,
    last_ip INET,
    last_user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

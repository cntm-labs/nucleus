# Phase 2: Complete Stub Handlers — Production Readiness

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Wire 4 stub auth handlers (MFA verify, magic link, OTP, password reset) to their existing service layers, implement dashboard API for the 5 repository-backed handler groups, and move all hardcoded config values to environment variables.

**Architecture:** Auth handlers already have fully tested service layers (mfa.rs, magic_link.rs, otp.rs, password_reset.rs) — the work is wiring handlers to services + repositories. Dashboard handlers already have proper routes — replace stub JSON with real DB queries using existing repositories. Config changes are additive to the existing Config struct.

**Tech Stack:** Rust (stable), Axum handlers, SQLx (Postgres), Redis, AES-256-GCM

**Execution order:** Config first (unblocks other tasks) → Auth handlers (highest user impact) → Dashboard API (admin panel)

---

## Task 1: Move Hardcoded Values to Config

### Problem
`main.rs` has hardcoded issuer URL (`"https://nucleus.local"`), JWT lifetime (`300`), and signing key ID. `passkey.rs` has hardcoded RP name/ID (`"Nucleus"`, `"localhost"`). `rate_limit.rs` has hardcoded thresholds. `well_known.rs` duplicates the issuer.

### Files
- Modify: `crates/nucleus-server/src/config.rs` — add new fields
- Modify: `crates/nucleus-server/src/main.rs` — use Config values
- Modify: `crates/nucleus-server/src/state.rs` — add issuer to AppState
- Modify: `crates/nucleus-auth/src/handlers/passkey.rs` — use config for RP name/ID
- Modify: `crates/nucleus-server/src/handlers/well_known.rs` — use issuer from state
- Modify: `crates/nucleus-server/src/middleware/rate_limit.rs` — use config for thresholds

### Step 1: Add fields to Config struct

In `crates/nucleus-server/src/config.rs`, add after existing fields:

```rust
pub issuer_url: String,
pub jwt_lifetime_secs: u64,
pub rp_name: String,
pub rp_id: String,
pub rate_limit_auth_max: u64,
pub rate_limit_auth_window_secs: u64,
pub rate_limit_api_max: u64,
pub rate_limit_api_window_secs: u64,
```

Load from env with defaults:

```rust
issuer_url: env::var("ISSUER_URL").unwrap_or_else(|_| "https://nucleus.local".to_string()),
jwt_lifetime_secs: env::var("JWT_LIFETIME_SECS")
    .ok().and_then(|v| v.parse().ok()).unwrap_or(300),
rp_name: env::var("RP_NAME").unwrap_or_else(|_| "Nucleus".to_string()),
rp_id: env::var("RP_ID").unwrap_or_else(|_| "localhost".to_string()),
rate_limit_auth_max: env::var("RATE_LIMIT_AUTH_MAX")
    .ok().and_then(|v| v.parse().ok()).unwrap_or(60),
rate_limit_auth_window_secs: env::var("RATE_LIMIT_AUTH_WINDOW_SECS")
    .ok().and_then(|v| v.parse().ok()).unwrap_or(60),
rate_limit_api_max: env::var("RATE_LIMIT_API_MAX")
    .ok().and_then(|v| v.parse().ok()).unwrap_or(1000),
rate_limit_api_window_secs: env::var("RATE_LIMIT_API_WINDOW_SECS")
    .ok().and_then(|v| v.parse().ok()).unwrap_or(60),
```

### Step 2: Add issuer_url to AppState

In `state.rs`, add `pub issuer_url: String` and update `AppState::new()`.

### Step 3: Update main.rs to use Config values

Replace hardcoded values in `main.rs`:

```rust
// OLD: "https://nucleus.local".to_string(), 300
// NEW:
let auth_service = Arc::new(AuthService::new(
    user_repo.clone(),
    credential_repo,
    audit_repo,
    signing_key.clone(),
    config.issuer_url.clone(),
    config.jwt_lifetime_secs,
));
```

### Step 4: Update well_known.rs

Replace hardcoded issuer in `handle_openid_config`:

```rust
// Extract issuer from AppState instead of hardcoded string
let issuer = &state.issuer_url;
```

### Step 5: Update passkey.rs

Replace hardcoded RP values — accept config values via handler state or as parameters.

### Step 6: Update rate_limit.rs

Replace `RateLimitConfig::auth()` and `RateLimitConfig::api()` to accept configurable values, or make them accept params from Config.

### Step 7: Update .env.example

Add all new env vars with sane defaults and comments.

### Step 8: Verify and commit

Run: `cargo test --workspace`
Run: `cargo clippy --workspace -- -D warnings`

```bash
git commit -m "refactor(config): move hardcoded issuer, JWT lifetime, RP, and rate limits to env config"
```

---

## Task 2: Create VerificationTokenRepository

### Problem
Magic link and password reset handlers need a `VerificationTokenRepository` to store/retrieve token hashes with expiry. The `verification_tokens` table exists in migration 006 but has no repository.

### Files
- Create: `crates/nucleus-db/src/repos/verification_token_repo.rs`
- Modify: `crates/nucleus-db/src/repos/mod.rs` — export new repo

### Step 1: Write the repository

```rust
// crates/nucleus-db/src/repos/verification_token_repo.rs

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
use nucleus_core::error::AppError;

pub struct VerificationToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub project_id: Uuid,
    pub token_hash: String,
    pub purpose: String, // "magic_link", "password_reset", "email_verify"
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
pub trait VerificationTokenRepository: Send + Sync {
    async fn create(
        &self,
        user_id: &Uuid,
        project_id: &Uuid,
        token_hash: &str,
        purpose: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<VerificationToken, AppError>;

    async fn find_by_hash(
        &self,
        token_hash: &str,
        purpose: &str,
    ) -> Result<Option<VerificationToken>, AppError>;

    async fn mark_used(&self, id: &Uuid) -> Result<(), AppError>;
}

pub struct PgVerificationTokenRepository {
    pool: PgPool,
}

impl PgVerificationTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VerificationTokenRepository for PgVerificationTokenRepository {
    async fn create(
        &self,
        user_id: &Uuid,
        project_id: &Uuid,
        token_hash: &str,
        purpose: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<VerificationToken, AppError> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        sqlx::query!(
            r#"INSERT INTO verification_tokens (id, user_id, project_id, token_hash, purpose, expires_at, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            id, user_id, project_id, token_hash, purpose, expires_at, now
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        Ok(VerificationToken {
            id, user_id: *user_id, project_id: *project_id,
            token_hash: token_hash.to_string(), purpose: purpose.to_string(),
            expires_at, used_at: None, created_at: now,
        })
    }

    async fn find_by_hash(
        &self,
        token_hash: &str,
        purpose: &str,
    ) -> Result<Option<VerificationToken>, AppError> {
        let row = sqlx::query_as!(
            VerificationToken,
            r#"SELECT id, user_id, project_id, token_hash, purpose, expires_at, used_at, created_at
               FROM verification_tokens WHERE token_hash = $1 AND purpose = $2"#,
            token_hash, purpose
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
        Ok(row)
    }

    async fn mark_used(&self, id: &Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE verification_tokens SET used_at = now() WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
        Ok(())
    }
}
```

### Step 2: Export from mod.rs

Add `pub mod verification_token_repo;` to `crates/nucleus-db/src/repos/mod.rs`.

### Step 3: Verify and commit

Run: `cargo check --workspace`

```bash
git commit -m "feat(db): add VerificationTokenRepository for magic link and password reset"
```

---

## Task 3: Wire Magic Link Verify Handler

### Problem
`handle_verify_magic_link()` in `handlers/magic_link.rs` returns `Err` with a comment saying it needs `VerificationTokenRepository`.

### Files
- Modify: `crates/nucleus-auth/src/handlers/magic_link.rs` — wire to service + repo
- Modify: `crates/nucleus-server/src/router.rs` — inject dependencies
- Test: add handler test

### Step 1: Implement handle_verify_magic_link

Replace the stub with real implementation using the existing `MagicLinkService::verify_token()`:

```rust
pub async fn handle_verify_magic_link(
    State(state): State<MagicLinkState>,
    Json(body): Json<VerifyMagicLinkRequest>,
) -> Result<impl IntoResponse, AppError> {
    let token_hash = crypto::hash_token(&body.token);
    let stored = state.token_repo
        .find_by_hash(&token_hash, "magic_link")
        .await?
        .ok_or(AppError::Auth(AuthError::TokenInvalid))?;

    MagicLinkService::verify_token(
        &body.token,
        &stored.token_hash,
        stored.expires_at,
        stored.used_at,
    )?;

    // Mark as used
    state.token_repo.mark_used(&stored.id).await?;

    // Find user and create session
    let user = state.user_repo
        .find_by_id(&UserId::from(stored.user_id))
        .await?
        .ok_or(AppError::Auth(AuthError::UserNotFound))?;

    let session = state.session_service.create_session(
        &UserId::from(stored.user_id),
        &ProjectId::from(stored.project_id),
        None, None, None,
    ).await?;

    let jwt = state.auth_service.issue_jwt(&user, &session)?;

    Ok(Json(json!({
        "user": user,
        "session": { "id": session.id, "token": jwt.token, "expires_at": jwt.expires_at },
    })))
}
```

### Step 2: Define MagicLinkState struct

```rust
pub struct MagicLinkState {
    pub token_repo: Arc<dyn VerificationTokenRepository>,
    pub user_repo: Arc<dyn UserRepository>,
    pub session_service: Arc<SessionService>,
    pub auth_service: Arc<AuthService>,
}
```

### Step 3: Wire in router.rs

Inject `MagicLinkState` into the magic link routes.

### Step 4: Verify and commit

Run: `cargo test --workspace`

```bash
git commit -m "feat(auth): wire magic link verification handler"
```

---

## Task 4: Wire Password Reset Confirm Handler

### Problem
`handle_confirm_reset()` returns success but never actually validates the token or changes the password.

### Files
- Modify: `crates/nucleus-auth/src/handlers/password_reset.rs` — wire to service + repo
- Test: add handler test

### Step 1: Implement handle_confirm_reset

Replace the stub with real implementation:

```rust
pub async fn handle_confirm_reset(
    State(state): State<PasswordResetState>,
    Json(body): Json<ConfirmResetRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate new password
    validation::validate_password(&body.new_password)?;

    // Find and verify token
    let token_hash = crypto::hash_token(&body.token);
    let stored = state.token_repo
        .find_by_hash(&token_hash, "password_reset")
        .await?
        .ok_or(AppError::Auth(AuthError::TokenInvalid))?;

    PasswordResetService::verify_token(
        &body.token,
        &stored.token_hash,
        stored.expires_at,
        stored.used_at,
    )?;

    // Mark token as used
    state.token_repo.mark_used(&stored.id).await?;

    // Hash new password and update
    let password_hash = crypto::hash_password(&body.new_password)?;
    state.credential_repo
        .update_password(&UserId::from(stored.user_id), &password_hash)
        .await?;

    // Revoke all sessions for security
    state.session_service
        .revoke_all_sessions(&UserId::from(stored.user_id))
        .await?;

    Ok(Json(json!({ "message": "Password reset successfully" })))
}
```

### Step 2: Define PasswordResetState and ConfirmResetRequest

```rust
pub struct PasswordResetState {
    pub token_repo: Arc<dyn VerificationTokenRepository>,
    pub credential_repo: Arc<dyn CredentialRepository>,
    pub session_service: Arc<SessionService>,
}

#[derive(Deserialize)]
pub struct ConfirmResetRequest {
    pub token: String,
    pub new_password: String,
}
```

### Step 3: Add update_password to CredentialRepository if missing

Check if `CredentialRepository` has an `update_password` method. If not, add it:

```rust
async fn update_password(&self, user_id: &UserId, password_hash: &str) -> Result<(), AppError>;
```

### Step 4: Verify and commit

Run: `cargo test --workspace`

```bash
git commit -m "feat(auth): wire password reset confirmation handler"
```

---

## Task 5: Wire OTP Verify Handler

### Problem
`handle_verify_otp()` returns `Err` — needs Redis integration for OTP storage/retrieval.

### Files
- Modify: `crates/nucleus-auth/src/handlers/otp.rs` — wire to OtpService + Redis
- Test: add handler test

### Step 1: Implement handle_verify_otp

Replace the stub. OTP codes are stored in Redis (key: `otp:{project}:{email_or_phone}:{purpose}`):

```rust
pub async fn handle_verify_otp(
    State(state): State<OtpState>,
    Json(body): Json<VerifyOtpRequest>,
) -> Result<impl IntoResponse, AppError> {
    let key = format!("otp:{}:{}:{}", body.project_id, body.email_or_phone, body.purpose);

    // Get stored OTP from Redis
    let stored: Option<StoredOtp> = state.redis_get(&key).await?;
    let stored = stored.ok_or(AppError::Auth(AuthError::OtpInvalid))?;

    // Verify using OtpService
    OtpService::verify(
        &body.code,
        &stored.code_hash,
        stored.expires_at,
        stored.attempts,
        stored.max_attempts,
    )?;

    // Delete OTP from Redis (single-use)
    state.redis_del(&key).await?;

    // Find or create user, create session
    let user = state.user_repo
        .find_by_email(&body.email_or_phone)
        .await?
        .ok_or(AppError::Auth(AuthError::UserNotFound))?;

    let session = state.session_service.create_session(
        &user.id, &ProjectId::from(body.project_id.parse::<Uuid>()?),
        None, None, None,
    ).await?;

    let jwt = state.auth_service.issue_jwt(&user, &session)?;

    Ok(Json(json!({
        "user": user,
        "session": { "id": session.id, "token": jwt.token, "expires_at": jwt.expires_at },
    })))
}
```

### Step 2: Also wire handle_send_otp to actually store OTP in Redis

The send handler currently returns success but does nothing. Wire it to generate + store:

```rust
pub async fn handle_send_otp(
    State(state): State<OtpState>,
    Json(body): Json<SendOtpRequest>,
) -> Result<impl IntoResponse, AppError> {
    let generated = OtpService::generate(OtpConfig::default());
    let key = format!("otp:{}:{}:{}", body.project_id, body.email_or_phone, body.purpose);

    let stored = StoredOtp {
        code_hash: generated.code_hash,
        expires_at: generated.expires_at,
        attempts: 0,
        max_attempts: 3,
    };
    state.redis_set(&key, &stored, 300).await?;

    // TODO: Send code via email/SMS service (not wired yet)
    // For now, log the code in development mode
    tracing::debug!("OTP code for {}: {}", body.email_or_phone, generated.code);

    // Always return success (anti-enumeration)
    Ok(Json(json!({ "message": "If the account exists, a code has been sent" })))
}
```

### Step 3: Verify and commit

Run: `cargo test --workspace`

```bash
git commit -m "feat(auth): wire OTP send and verify handlers with Redis storage"
```

---

## Task 6: Wire MFA TOTP Verify Handler

### Problem
`handle_mfa_totp_verify()` and `handle_mfa_verify()` return `Err`. The `MfaService` is fully implemented but handlers don't call it.

### Files
- Modify: `crates/nucleus-auth/src/handlers/mfa.rs` — wire to MfaService
- Test: add handler test

### Step 1: Create MfaEnrollmentRepository

Need a repo for `mfa_enrollments` table to store TOTP secrets per user:

```rust
// crates/nucleus-db/src/repos/mfa_enrollment_repo.rs

#[async_trait]
pub trait MfaEnrollmentRepository: Send + Sync {
    async fn create(&self, user_id: &Uuid, project_id: &Uuid, method: &str, secret_enc: &str, backup_codes_enc: &str) -> Result<MfaEnrollment, AppError>;
    async fn find_active(&self, user_id: &Uuid, project_id: &Uuid, method: &str) -> Result<Option<MfaEnrollment>, AppError>;
    async fn update_backup_codes(&self, id: &Uuid, backup_codes_enc: &str) -> Result<(), AppError>;
    async fn delete(&self, id: &Uuid) -> Result<(), AppError>;
}
```

### Step 2: Implement handle_mfa_totp_verify

```rust
pub async fn handle_mfa_totp_verify(
    State(state): State<MfaState>,
    JwtAuth(claims): JwtAuth,
    Json(body): Json<MfaTotpVerifyRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = claims.sub.parse::<Uuid>()
        .map_err(|_| AppError::Validation("Invalid user ID".into()))?;
    let project_id = claims.aud.parse::<Uuid>()
        .map_err(|_| AppError::Validation("Invalid project ID".into()))?;

    let enrollment = state.mfa_repo
        .find_active(&user_id, &project_id, "totp")
        .await?
        .ok_or(AppError::Auth(AuthError::MfaNotEnrolled))?;

    let verified = MfaService::verify_totp(
        &body.code,
        &enrollment.secret_enc,
        &state.master_key,
    )?;

    Ok(Json(json!({ "verified": verified })))
}
```

### Step 3: Implement handle_mfa_disable

```rust
pub async fn handle_mfa_disable(
    State(state): State<MfaState>,
    JwtAuth(claims): JwtAuth,
) -> Result<impl IntoResponse, AppError> {
    let user_id = claims.sub.parse::<Uuid>()?;
    let project_id = claims.aud.parse::<Uuid>()?;

    let enrollment = state.mfa_repo
        .find_active(&user_id, &project_id, "totp")
        .await?
        .ok_or(AppError::Auth(AuthError::MfaNotEnrolled))?;

    state.mfa_repo.delete(&enrollment.id).await?;

    Ok(Json(json!({ "message": "MFA disabled" })))
}
```

### Step 4: Verify and commit

Run: `cargo test --workspace`

```bash
git commit -m "feat(auth): wire MFA TOTP verify and disable handlers"
```

---

## Task 7: Implement Dashboard API — Projects, API Keys, Audit Logs

### Problem
All 26 dashboard handlers return fake data. This task covers the 3 groups that have existing repositories: Projects (4 handlers), API Keys (3 handlers), Audit Logs (1 handler).

### Files
- Modify: `crates/nucleus-admin-api/src/handlers/dashboard.rs`
- Modify: `crates/nucleus-server/src/router.rs` — inject repos into dashboard state

### Step 1: Define DashboardState

```rust
pub struct DashboardState {
    pub project_repo: Arc<dyn ProjectRepository>,
    pub api_key_repo: Arc<dyn ApiKeyRepository>,
    pub audit_repo: Arc<dyn AuditRepository>,
    pub signing_key_repo: Arc<dyn SigningKeyRepository>,
    pub master_key: [u8; 32],
}
```

### Step 2: Implement handle_list_projects

```rust
pub async fn handle_list_projects(
    State(state): State<DashboardState>,
    // TODO: Extract account_id from session
) -> Result<impl IntoResponse, AppError> {
    let projects = state.project_repo.list_by_account(&account_id).await?;
    Ok(Json(json!({
        "data": projects,
        "has_more": false,
        "total_count": projects.len(),
    })))
}
```

### Step 3: Implement handle_get_project, handle_create_project, handle_update_project

Wire each to the corresponding `ProjectRepository` method.

### Step 4: Implement handle_list_api_keys, handle_create_api_key, handle_revoke_api_key

Wire to `ApiKeyRepository`. For create, generate a real key, hash it, store, return full key once.

### Step 5: Implement handle_list_audit_logs

Wire to `AuditRepository.list_audit_logs()`.

### Step 6: Verify and commit

Run: `cargo test --workspace`

```bash
git commit -m "feat(dashboard): implement projects, API keys, and audit logs with real DB queries"
```

---

## Task 8: Implement Dashboard API — Settings and Signing Keys

### Problem
Settings and signing key handlers return hardcoded defaults instead of querying the project's actual settings.

### Files
- Modify: `crates/nucleus-admin-api/src/handlers/dashboard.rs` — settings + signing key handlers
- Modify: `crates/nucleus-db/src/repos/signing_key_repo.rs` — add list method

### Step 1: Add list_by_project to SigningKeyRepository

```rust
async fn list_by_project(&self, project_id: &Uuid) -> Result<Vec<SigningKeyRow>, AppError>;
```

Implement:

```rust
async fn list_by_project(&self, project_id: &Uuid) -> Result<Vec<SigningKeyRow>, AppError> {
    let rows = sqlx::query_as!(
        SigningKeyRow,
        "SELECT id, project_id, algorithm, public_key, private_key_enc, is_current
         FROM signing_keys WHERE project_id = $1 ORDER BY created_at DESC",
        project_id
    )
    .fetch_all(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    Ok(rows)
}
```

### Step 2: Implement handle_get_settings

```rust
pub async fn handle_get_settings(
    State(state): State<DashboardState>,
    Path(project_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let project = state.project_repo
        .find_by_id(&ProjectId::from(project_id))
        .await?
        .ok_or(AppError::NotFound("Project not found".into()))?;

    Ok(Json(json!({
        "session_ttl": project.session_ttl,
        "jwt_lifetime": project.jwt_lifetime,
        "jwt_algorithm": project.jwt_algorithm,
        "allowed_origins": project.allowed_origins,
        "settings": project.settings,
    })))
}
```

### Step 3: Implement handle_update_settings

Merge submitted JSON into existing settings, call `project_repo.update_settings()`.

### Step 4: Implement handle_list_signing_keys and handle_rotate_signing_key

Wire to `SigningKeyRepository`. For rotate: generate new key, encrypt private key, mark old as inactive, insert new.

### Step 5: Verify and commit

Run: `cargo test --workspace`

```bash
git commit -m "feat(dashboard): implement settings and signing key management"
```

---

## Verification Checklist

After all tasks complete:

```bash
# Full test suite
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all -- --check

# Verify no stub Err returns remain in auth handlers
grep -rn "Requires.*Repository\|TODO.*wire\|placeholder" crates/nucleus-auth/src/handlers/ --include="*.rs"

# Verify no hardcoded issuer
grep -rn '"https://nucleus.local"' crates/ --include="*.rs" | grep -v test | grep -v config

# Verify dashboard handlers don't return fabricated data
grep -rn "Uuid::new_v4()\|\"fake\"\|hardcoded" crates/nucleus-admin-api/ --include="*.rs" | grep -v test
```

All must pass before PR.

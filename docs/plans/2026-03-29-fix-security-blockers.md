# Phase 1: Security Blockers — Production Readiness

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix 7 critical security issues that block production deployment — JWT key persistence, session token binding, JWT revocation enforcement, OAuth project isolation, sign-out authentication, webhook secret encryption, and org route authentication.

**Architecture:** Each fix is isolated to 1-2 crates. Tasks are ordered by dependency — JWT signing keys must be persisted first (Task 1) because other tasks depend on a working auth stack. The middleware-based approach (Tasks 5, 7) follows a consistent pattern: extract context from JWT claims instead of trusting query parameters.

**Tech Stack:** Rust (stable), Axum middleware, SQLx (Postgres), Redis, AES-256-GCM encryption, RS256 JWT

**Execution order:** JWT keys → Session binding → JWT revocation → OAuth project_id → Sign-out auth → Webhook encryption → Org route auth

---

## Task 1: Persist JWT Signing Keys

### Problem
`main.rs:57` generates a fresh RSA key pair on every server startup. All previously issued JWTs become invalid on restart. The `signing_keys` table exists (migration 013) but is never used.

### Files
- Create: `crates/nucleus-db/src/repos/signing_key_repo.rs`
- Modify: `crates/nucleus-db/src/repos/mod.rs` — export new repo
- Modify: `crates/nucleus-server/src/main.rs` — load or create key from DB
- Modify: `crates/nucleus-server/src/state.rs` — no changes needed (already holds `signing_key`)
- Test: `crates/nucleus-db/src/repos/signing_key_repo.rs` (inline tests)

### Step 1: Create SigningKeyRepository trait and Postgres implementation

```rust
// crates/nucleus-db/src/repos/signing_key_repo.rs

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use nucleus_core::error::AppError;

pub struct SigningKeyRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub algorithm: String,
    pub public_key: String,
    pub private_key_enc: String,
    pub is_current: bool,
}

#[async_trait]
pub trait SigningKeyRepository: Send + Sync {
    async fn find_current(&self, project_id: &Uuid) -> Result<Option<SigningKeyRow>, AppError>;
    async fn create(&self, project_id: &Uuid, algorithm: &str, public_key: &str, private_key_enc: &str) -> Result<SigningKeyRow, AppError>;
}

pub struct PgSigningKeyRepository {
    pool: PgPool,
}

impl PgSigningKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SigningKeyRepository for PgSigningKeyRepository {
    async fn find_current(&self, project_id: &Uuid) -> Result<Option<SigningKeyRow>, AppError> {
        let row = sqlx::query_as!(
            SigningKeyRow,
            r#"SELECT id, project_id, algorithm, public_key, private_key_enc, is_current
               FROM signing_keys WHERE project_id = $1 AND is_current = true"#,
            project_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
        Ok(row)
    }

    async fn create(&self, project_id: &Uuid, algorithm: &str, public_key: &str, private_key_enc: &str) -> Result<SigningKeyRow, AppError> {
        let id = Uuid::new_v4();
        sqlx::query!(
            r#"INSERT INTO signing_keys (id, project_id, algorithm, public_key, private_key_enc, is_current)
               VALUES ($1, $2, $3, $4, $5, true)"#,
            id, project_id, algorithm, public_key, private_key_enc
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        Ok(SigningKeyRow { id, project_id: *project_id, algorithm: algorithm.to_string(), public_key: public_key.to_string(), private_key_enc: private_key_enc.to_string(), is_current: true })
    }
}
```

### Step 2: Export from mod.rs

Add `pub mod signing_key_repo;` to `crates/nucleus-db/src/repos/mod.rs`.

### Step 3: Modify main.rs — load or create key

Replace lines 57-60 in `main.rs`:

```rust
// OLD:
// let signing_key = JwtService::generate_key_pair("nucleus-key-1")?;

// NEW:
let signing_key_repo = PgSigningKeyRepository::new(db.clone());
let default_project_id = Uuid::nil(); // System-level key for now
let signing_key = match signing_key_repo.find_current(&default_project_id).await? {
    Some(row) => {
        let private_pem = crypto::decrypt(
            &hex::decode(&row.private_key_enc).map_err(|e| anyhow::anyhow!(e))?,
            &config.master_encryption_key,
        )?;
        SigningKeyPair {
            kid: row.id.to_string(),
            private_key_pem: private_pem,
            public_key_pem: row.public_key.as_bytes().to_vec(),
            algorithm: jsonwebtoken::Algorithm::RS256,
        }
    }
    None => {
        let key = JwtService::generate_key_pair(&Uuid::new_v4().to_string())?;
        let encrypted = hex::encode(crypto::encrypt(&key.private_key_pem, &config.master_encryption_key)?);
        let public_pem = String::from_utf8(key.public_key_pem.clone()).unwrap();
        signing_key_repo.create(&default_project_id, "RS256", &public_pem, &encrypted).await?;
        key
    }
};
```

### Step 4: Verify

Run: `cargo test --workspace`
Run: `cargo clippy --workspace -- -D warnings`

### Step 5: Commit

```bash
git commit -m "fix(server): persist JWT signing keys in database with AES-256-GCM encryption"
```

---

## Task 2: Bind Session Token to Stored Session

### Problem
`session.rs:26-47` generates a 256-bit session token but never stores or verifies it. Session lookup uses `SessionId` (an unprotected UUID). Anyone who guesses/observes a session ID can hijack the session.

### Files
- Modify: `crates/nucleus-session/src/session.rs` — store token hash, verify on validate
- Modify: `crates/nucleus-db/src/repos/session_repo.rs` — add token_hash to Session struct and Redis storage
- Test: existing tests in `session.rs` will be updated

### Step 1: Add token_hash to Session and NewSession

In `crates/nucleus-db/src/repos/session_repo.rs`, add `token_hash: String` to the `Session` struct and `NewSession` struct.

### Step 2: Store token_hash in Redis

In `RedisSessionRepository::create()`, store `token_hash` in the Redis hash alongside other session fields.

### Step 3: Add verify method to SessionService

In `crates/nucleus-session/src/session.rs`, add:

```rust
pub async fn validate_session_with_token(
    &self,
    session_id: &SessionId,
    token: &str,
) -> Result<Session, AppError> {
    let session = self.validate_session(session_id).await?;
    let token_hash = crypto::hash_token(token);
    if !crypto::constant_time_eq(token_hash.as_bytes(), session.token_hash.as_bytes()) {
        return Err(AppError::Auth(AuthError::SessionInvalid));
    }
    Ok(session)
}
```

### Step 4: Hash token before storing in create_session

In `SessionService::create_session()`, hash the generated token and pass to `NewSession`:

```rust
let token = crypto::generate_token();
let token_hash = crypto::hash_token(&token);
let new_session = NewSession {
    // ... existing fields ...
    token_hash,
};
```

### Step 5: Update tests

Update existing session tests to verify token binding works correctly.

### Step 6: Verify and commit

Run: `cargo test --workspace`

```bash
git commit -m "fix(session): bind session token hash to stored session for validation"
```

---

## Task 3: Wire JWT Revocation Check in Auth Middleware

### Problem
`JwtAuth` middleware in `auth.rs` verifies the JWT signature and expiry but never checks if the JWT has been revoked (via `is_jwt_revoked`). A revoked token (from sign-out) remains usable until its 5-minute expiry.

### Files
- Modify: `crates/nucleus-server/src/middleware/auth.rs` — add revocation check
- Modify: `crates/nucleus-server/src/state.rs` — may need SessionService access in extractor
- Test: `crates/nucleus-server/src/middleware/auth.rs` (add test)

### Step 1: Add revocation check to JwtAuth extractor

After JWT verification succeeds (line 62 of `auth.rs`), check the revocation list:

```rust
// After: let claims = JwtService::verify(token, &public_key.0, &project_id.to_string())?;

// Add revocation check
let session_service = parts
    .extensions
    .get::<Arc<SessionService>>()
    .ok_or(AppError::Internal(anyhow::anyhow!("SessionService not in extensions")))?;

if session_service.is_jwt_revoked(&claims.jti).await? {
    return Err(AppError::Auth(AuthError::TokenRevoked));
}
```

### Step 2: Add SessionService to request extensions

In `router.rs`, add a middleware layer that inserts `Arc<SessionService>` into request extensions:

```rust
async fn inject_session_service(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Response {
    req.extensions_mut().insert(state.session_service.clone());
    next.run(req).await
}
```

Add this layer to the router before auth routes.

### Step 3: Add TokenRevoked variant to AuthError

In `crates/nucleus-core/src/error.rs`, add:

```rust
TokenRevoked, // in AuthError enum
```

Map it to 401 status with code `"auth/token_revoked"`.

### Step 4: Verify and commit

Run: `cargo test --workspace`

```bash
git commit -m "fix(auth): check JWT revocation list in auth middleware"
```

---

## Task 4: Fix OAuth project_id

### Problem
`oauth.rs:122` creates a random `ProjectId::new()` for every OAuth flow instead of using the caller's actual project. OAuth users are never associated with the correct project.

### Files
- Modify: `crates/nucleus-auth/src/handlers/oauth.rs` — extract project_id from request context
- Test: update existing OAuth handler tests

### Step 1: Extract project_id from request extensions or API key

Replace line 122:

```rust
// OLD:
// let project_id = ProjectId::new();

// NEW: Extract from request extensions (set by API key middleware or project lookup)
let project_id = parts
    .extensions
    .get::<ProjectId>()
    .cloned()
    .ok_or(AppError::Auth(AuthError::TokenInvalid))?;
```

For the `handle_oauth_start` function, accept `ProjectId` as an extractor parameter since it should be set by the publishable key middleware.

### Step 2: Update handle_oauth_callback

The callback extracts `project_id` from the stored OAuth state (which was saved during `handle_oauth_start`). This already works correctly — the state stores `project_id` from the start flow. The fix is only needed in `handle_oauth_start`.

### Step 3: Update tests

Update the `test_oauth_start_url` and related tests to inject a `ProjectId` extension.

### Step 4: Verify and commit

Run: `cargo test --workspace`

```bash
git commit -m "fix(oauth): use caller's project_id instead of random UUID"
```

---

## Task 5: Fix Sign-Out Endpoints to Require Authentication

### Problem
`token.rs:60-79` and `token.rs:95-110` accept `user_id` and `session_id` from the JSON request body without requiring JWT authentication. Any client can sign out any user.

### Files
- Modify: `crates/nucleus-auth/src/handlers/token.rs` — use JwtAuth extractor
- Modify: `crates/nucleus-server/src/router.rs` — ensure auth middleware is applied
- Test: add test for unauthorized sign-out rejection

### Step 1: Modify handle_sign_out to use JwtAuth

```rust
// OLD:
pub async fn handle_sign_out(
    State(session_service): State<Arc<SessionService>>,
    Json(body): Json<SignOutRequest>,
) -> Result<impl IntoResponse, AppError> {
    let session_id = SessionId::from(Uuid::parse_str(&body.session_id)...);
    let user_id = UserId::from(Uuid::parse_str(&body.user_id)...);

// NEW:
pub async fn handle_sign_out(
    JwtAuth(claims): JwtAuth,
    State(session_service): State<Arc<SessionService>>,
) -> Result<impl IntoResponse, AppError> {
    let session_id = SessionId::from(claims.sid.parse::<Uuid>()
        .map_err(|_| AppError::Validation("Invalid session ID".into()))?);
    let user_id = UserId::from(claims.sub.parse::<Uuid>()
        .map_err(|_| AppError::Validation("Invalid user ID".into()))?);
```

### Step 2: Same for handle_sign_out_all

Extract `user_id` from `JwtAuth(claims)` instead of the request body.

### Step 3: Same for handle_refresh

Extract `session_id` from JWT claims instead of request body.

### Step 4: Ensure PublicKeyPem and ProjectId extensions are set for auth routes

Add a middleware layer in `router.rs` that injects `PublicKeyPem` and `ProjectId` from `AppState.signing_key` for auth routes. For now, use the system-level key:

```rust
async fn inject_auth_context(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Response {
    req.extensions_mut().insert(PublicKeyPem(state.signing_key.public_key_pem.clone()));
    req.extensions_mut().insert(ProjectId::from(Uuid::nil())); // System project
    req.extensions_mut().insert(state.session_service.clone());
    next.run(req).await
}
```

Apply to auth routes that need JWT validation (sign-out, refresh, user profile).

### Step 5: Verify and commit

Run: `cargo test --workspace`

```bash
git commit -m "fix(auth): require JWT authentication for sign-out and refresh endpoints"
```

---

## Task 6: Encrypt Webhook Secret at Rest

### Problem
`projects.webhook_secret` is stored as plaintext in the `projects` table. The schema supports encryption (other columns use `_enc` suffix) but webhook secrets are not encrypted.

### Files
- Modify: `crates/nucleus-db/src/repos/project_repo.rs` — encrypt on write, decrypt on read
- Modify: Migration: add `ALTER TABLE projects RENAME COLUMN webhook_secret TO webhook_secret_enc`
- Create: `crates/nucleus-migrate/migrations/014_encrypt_webhook_secret.sql`
- Modify: `crates/nucleus-webhook/src/dispatcher.rs` — use constant-time comparison
- Test: update project repo tests

### Step 1: Add migration to rename column

```sql
-- crates/nucleus-migrate/migrations/014_encrypt_webhook_secret.sql
ALTER TABLE projects RENAME COLUMN webhook_secret TO webhook_secret_enc;
```

### Step 2: Update Project struct

In `project_repo.rs`, rename `webhook_secret` to `webhook_secret_enc` in the `Project` struct.

### Step 3: Add encrypt/decrypt helpers to project repo

```rust
use nucleus_core::crypto;

impl PgProjectRepository {
    pub fn encrypt_webhook_secret(secret: &str, master_key: &[u8; 32]) -> Result<String, AppError> {
        let encrypted = crypto::encrypt(secret.as_bytes(), master_key)?;
        Ok(hex::encode(encrypted))
    }

    pub fn decrypt_webhook_secret(encrypted_hex: &str, master_key: &[u8; 32]) -> Result<String, AppError> {
        let encrypted = hex::decode(encrypted_hex).map_err(|e| AppError::Internal(e.into()))?;
        let decrypted = crypto::decrypt(&encrypted, master_key)?;
        String::from_utf8(decrypted).map_err(|e| AppError::Internal(e.into()))
    }
}
```

### Step 4: Fix webhook signature to use constant-time comparison

In `dispatcher.rs:39`, replace:

```rust
// OLD:
expected == signature

// NEW:
crypto::constant_time_eq(expected.as_bytes(), signature.as_bytes())
```

### Step 5: Verify and commit

Run: `cargo test --workspace`

```bash
git commit -m "fix(webhook): encrypt webhook_secret at rest and use constant-time signature comparison"
```

---

## Task 7: Add Authentication to Org Routes

### Problem
All org routes accept `project_id` and `user_id` as query parameters with no authentication. Any caller can impersonate any user in any organization.

### Files
- Modify: `crates/nucleus-server/src/handlers/org.rs` — use JwtAuth extractor
- Modify: `crates/nucleus-org/src/handlers/org.rs` — accept UserId/ProjectId from auth context
- Modify: `crates/nucleus-org/src/handlers/member.rs` — same
- Modify: `crates/nucleus-org/src/handlers/invitation.rs` — same
- Modify: `crates/nucleus-server/src/router.rs` — add auth middleware to org routes

### Step 1: Add auth middleware layer to org routes in router.rs

```rust
let org_routes = Router::new()
    .route("/", get(org::handle_list_orgs).post(org::handle_create_org))
    // ... other org routes ...
    .layer(axum::middleware::from_fn_with_state(
        state.clone(),
        inject_auth_context,
    ))
    .layer(axum::middleware::from_fn(api_rate_limiter));
```

### Step 2: Update server org handlers to use JwtAuth

In `crates/nucleus-server/src/handlers/org.rs`, replace query param extraction with JWT claims:

```rust
pub async fn handle_list_orgs(
    JwtAuth(claims): JwtAuth,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let project_id = ProjectId::from(claims.aud.parse::<Uuid>()
        .map_err(|_| AppError::Validation("Invalid project ID".into()))?);
    let user_id = UserId::from(claims.sub.parse::<Uuid>()
        .map_err(|_| AppError::Validation("Invalid user ID".into()))?);
    // ... delegate to nucleus-org handler with verified IDs ...
}
```

### Step 3: Update nucleus-org handlers to accept typed IDs

Change `RequestContext` and `ProjectContext` to use `ProjectId` and `UserId` types instead of raw strings. Update all handler signatures.

### Step 4: Remove query param-based context structs

Delete `RequestContext { project_id: String, user_id: String }` and replace with direct extractor parameters.

### Step 5: Verify and commit

Run: `cargo test --workspace`

```bash
git commit -m "fix(org): require JWT authentication for all organization routes"
```

---

## Verification Checklist

After all tasks complete:

```bash
# Full test suite
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings

# Format check
cargo fmt --all -- --check

# Verify no hardcoded project_id in handlers
grep -rn "ProjectId::new()" crates/ --include="*.rs" | grep -v test | grep -v mock

# Verify no unauthed body-based user_id extraction in handlers
grep -rn "body.user_id\|body.session_id" crates/ --include="*.rs" | grep -v test

# Verify webhook_secret renamed to webhook_secret_enc
grep -rn "webhook_secret[^_]" crates/ --include="*.rs" | grep -v test | grep -v "_enc"
```

All must pass before PR.

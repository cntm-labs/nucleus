# Nucleus Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build Nucleus — a production-grade authentication SaaS platform (like Clerk) with a Rust backend, 14 SDKs, and an admin dashboard.

**Architecture:** Modular Monolith in Rust (Axum). PostgreSQL for persistence, Redis for sessions + caching. Hybrid JWT + Redis sessions. Shared-schema multi-tenancy. 14 SDKs across frontend, mobile, and backend platforms.

**Tech Stack:** Rust (Axum, sqlx, ring, argon2, jsonwebtoken), PostgreSQL 16, Redis 7, React (Vite), TypeScript, Dart (Flutter), Swift, Kotlin, Java, Go, Python, C#

**Design Doc:** `docs/plans/2026-03-19-nucleus-design.md`

---

## Phase 1: Foundation — Workspace, Core, Database

### Task 1.1: Scaffold Cargo Workspace

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `crates/nucleus-core/Cargo.toml`
- Create: `crates/nucleus-core/src/lib.rs`
- Create: `crates/nucleus-db/Cargo.toml`
- Create: `crates/nucleus-db/src/lib.rs`
- Create: `crates/nucleus-server/Cargo.toml`
- Create: `crates/nucleus-server/src/main.rs`
- Create: `crates/nucleus-auth/Cargo.toml`
- Create: `crates/nucleus-auth/src/lib.rs`
- Create: `crates/nucleus-identity/Cargo.toml`
- Create: `crates/nucleus-identity/src/lib.rs`
- Create: `crates/nucleus-org/Cargo.toml`
- Create: `crates/nucleus-org/src/lib.rs`
- Create: `crates/nucleus-session/Cargo.toml`
- Create: `crates/nucleus-session/src/lib.rs`
- Create: `crates/nucleus-webhook/Cargo.toml`
- Create: `crates/nucleus-webhook/src/lib.rs`
- Create: `crates/nucleus-admin-api/Cargo.toml`
- Create: `crates/nucleus-admin-api/src/lib.rs`
- Create: `crates/nucleus-migrate/Cargo.toml`
- Create: `crates/nucleus-migrate/src/lib.rs`
- Create: `rustfmt.toml`
- Create: `clippy.toml`
- Create: `deny.toml`
- Create: `.env.example`

**Step 1: Create workspace Cargo.toml with all workspace dependencies**

```toml
[workspace]
resolver = "2"
members = [
    "crates/nucleus-server",
    "crates/nucleus-core",
    "crates/nucleus-db",
    "crates/nucleus-auth",
    "crates/nucleus-identity",
    "crates/nucleus-org",
    "crates/nucleus-session",
    "crates/nucleus-webhook",
    "crates/nucleus-admin-api",
    "crates/nucleus-migrate",
]

[workspace.dependencies]
# Web
axum = "0.8"
axum-extra = { version = "0.10", features = ["typed-header"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "compression-gzip", "trace", "request-id", "util"] }

# Async
tokio = { version = "1", features = ["full"] }

# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono", "json", "migrate"] }
redis = { version = "0.27", features = ["tokio-comp", "connection-manager"] }

# Crypto
argon2 = "0.5"
aes-gcm = "0.10"
hmac = "0.12"
sha2 = "0.10"
ring = "0.17"
jsonwebtoken = "9"
webauthn-rs = { version = "0.5", features = ["danger-allow-state-serialisation"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Validation
validator = { version = "0.18", features = ["derive"] }

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
metrics = "0.24"
metrics-exporter-prometheus = "0.16"

# Config
dotenvy = "0.15"

# Time
chrono = { version = "0.4", features = ["serde"] }

# Utilities
uuid = { version = "1", features = ["v4", "serde"] }
thiserror = "2"
anyhow = "1"
url = "2"
base64 = "0.22"
rand = "0.8"
hex = "0.4"

# Testing
[workspace.dev-dependencies]
tokio-test = "0.4"
testcontainers = "0.23"
testcontainers-modules = { version = "0.11", features = ["postgres", "redis"] }
wiremock = "0.6"
criterion = { version = "0.5", features = ["async_tokio"] }
proptest = "1"

[profile.release]
strip = true
lto = true
codegen-units = 1
```

**Step 2: Create each crate's Cargo.toml with correct dependencies**

Each crate depends only on allowed crates per the dependency rules:
- `nucleus-core` → external deps only (leaf crate)
- `nucleus-db` → nucleus-core
- Feature crates → nucleus-core + nucleus-db
- `nucleus-server` → all crates

**Step 3: Create stub lib.rs/main.rs for each crate**

`nucleus-server/src/main.rs`:
```rust
#[tokio::main]
async fn main() {
    println!("Nucleus server starting...");
}
```

All other crates: empty `pub mod` stubs.

**Step 4: Create config files**

`rustfmt.toml`:
```toml
edition = "2021"
max_width = 100
use_field_init_shorthand = true
```

`clippy.toml`:
```toml
too-many-arguments-threshold = 8
```

`.env.example`:
```
DATABASE_URL=postgres://nucleus:nucleus@localhost:5432/nucleus
REDIS_URL=redis://localhost:6379
MASTER_ENCRYPTION_KEY=change-me-in-production-64-hex-chars
JWT_RSA_PRIVATE_KEY_PATH=./dev-keys/private.pem
HOST=0.0.0.0
PORT=3000
RUST_LOG=nucleus=debug,tower_http=debug
```

**Step 5: Verify workspace compiles**

Run: `cargo check --workspace`
Expected: compiles with no errors

**Step 6: Commit**

```bash
git add -A
git commit -m "feat: scaffold Cargo workspace with 10 crates"
```

---

### Task 1.2: nucleus-core — Error Types & Shared Types

**Files:**
- Create: `crates/nucleus-core/src/error.rs`
- Create: `crates/nucleus-core/src/types.rs`
- Create: `crates/nucleus-core/src/pagination.rs`
- Modify: `crates/nucleus-core/src/lib.rs`
- Test: inline `#[cfg(test)]` modules

**Step 1: Write failing tests for error types**

```rust
// error.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_formats_correctly() {
        let err = AppError::Auth(AuthError::InvalidCredentials);
        assert_eq!(err.code(), "auth/invalid_credentials");
        assert_eq!(err.status(), 401);
    }

    #[test]
    fn error_serializes_to_json() {
        let err = AppError::Auth(AuthError::InvalidCredentials);
        let json = serde_json::to_value(err.to_response("req_123")).unwrap();
        assert_eq!(json["error"]["code"], "auth/invalid_credentials");
        assert_eq!(json["error"]["status"], 401);
        assert_eq!(json["error"]["request_id"], "req_123");
    }
}
```

**Step 2: Run tests — verify they fail**

Run: `cargo test -p nucleus-core`
Expected: FAIL — types not defined

**Step 3: Implement error types**

Full error taxonomy from design doc:
- `AuthError` — InvalidCredentials, AccountLocked, AccountBanned, EmailNotVerified, MfaRequired, SessionExpired, SessionRevoked, TokenExpired, TokenInvalid, etc.
- `UserError` — NotFound, EmailTaken, UsernameTaken, InvalidEmail, UpdateForbidden
- `OrgError` — NotFound, SlugTaken, MemberAlreadyExists, MemberLimitReached, InsufficientPermissions, InvitationExpired
- `ApiError` — InvalidApiKey, KeyRevoked, KeyExpired, InsufficientScopes, RateLimited, ProjectSuspended, PlanLimitExceeded, ValidationError, InternalError

Implement `AppError` with `code()`, `status()`, `message()` methods. Implement `IntoResponse` for Axum.

**Step 4: Implement newtype IDs**

```rust
// types.rs
macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
        #[sqlx(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self { Self(Uuid::new_v4()) }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_id!(ProjectId);
define_id!(UserId);
define_id!(OrgId);
define_id!(SessionId);
define_id!(ApiKeyId);
define_id!(AccountId);
define_id!(RoleId);
define_id!(PermissionId);
define_id!(CredentialId);
define_id!(WebhookEventId);
```

**Step 5: Implement cursor-based pagination**

```rust
// pagination.rs
pub struct PaginationParams {
    pub limit: u32,      // max 100, default 20
    pub cursor: Option<String>,
}

pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
    pub total_count: Option<u64>,
}
```

**Step 6: Run tests — verify they pass**

Run: `cargo test -p nucleus-core`
Expected: ALL PASS

**Step 7: Commit**

```bash
git add crates/nucleus-core/
git commit -m "feat(core): add error types, ID newtypes, and pagination"
```

---

### Task 1.3: nucleus-core — Cryptography Primitives

**Files:**
- Create: `crates/nucleus-core/src/crypto.rs`
- Create: `crates/nucleus-core/src/validation.rs`
- Create: `crates/nucleus-core/src/clock.rs`
- Modify: `crates/nucleus-core/src/lib.rs`

**Step 1: Write failing tests for crypto**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_hash_roundtrip() {
        let hash = hash_password("my_secure_password").unwrap();
        assert!(verify_password("my_secure_password", &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn password_hash_unique_salt() {
        let h1 = hash_password("same_password").unwrap();
        let h2 = hash_password("same_password").unwrap();
        assert_ne!(h1, h2); // different salts
    }

    #[test]
    fn aes_gcm_encrypt_decrypt_roundtrip() {
        let key = generate_encryption_key();
        let plaintext = b"secret MFA data";
        let ciphertext = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&ciphertext, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn aes_gcm_rejects_tampered_ciphertext() {
        let key = generate_encryption_key();
        let mut ciphertext = encrypt(b"secret", &key).unwrap();
        ciphertext[10] ^= 0xFF; // tamper
        assert!(decrypt(&ciphertext, &key).is_err());
    }

    #[test]
    fn generate_token_is_256_bit() {
        let token = generate_token();
        assert_eq!(token.len(), 43); // 32 bytes base64url = 43 chars
    }

    #[test]
    fn hmac_sha256_sign_verify() {
        let key = b"webhook_secret";
        let payload = b"event data";
        let sig = hmac_sign(key, payload);
        assert!(hmac_verify(key, payload, &sig));
        assert!(!hmac_verify(key, b"tampered", &sig));
    }

    #[test]
    fn constant_time_compare_works() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
    }
}
```

**Step 2: Run tests — verify they fail**

**Step 3: Implement crypto module**

- `hash_password(password: &str) -> Result<String>` — Argon2id (19MB, 2 iterations, parallelism 1)
- `verify_password(password: &str, hash: &str) -> Result<bool>`
- `encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>>` — AES-256-GCM
- `decrypt(ciphertext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>>`
- `generate_encryption_key() -> [u8; 32]` — CSPRNG
- `generate_token() -> String` — 256-bit random, base64url
- `generate_token_hash(token: &str) -> String` — SHA-256
- `hmac_sign(key: &[u8], payload: &[u8]) -> String` — HMAC-SHA256 hex
- `hmac_verify(key: &[u8], payload: &[u8], signature: &str) -> bool`
- `constant_time_eq(a: &[u8], b: &[u8]) -> bool` — ring constant-time

**Step 4: Implement validation module**

- `validate_email(email: &str) -> Result<String>` — normalize + validate
- `validate_password(password: &str) -> Result<()>` — min 8, max 128
- `validate_url(url: &str) -> Result<Url>` — parse + reject private IPs
- `validate_slug(slug: &str) -> Result<()>` — alphanumeric + hyphens

**Step 5: Implement Clock trait**

```rust
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> { Utc::now() }
}

#[cfg(test)]
pub struct MockClock(pub DateTime<Utc>);
```

**Step 6: Run tests — verify they pass**

Run: `cargo test -p nucleus-core`
Expected: ALL PASS

**Step 7: Commit**

```bash
git add crates/nucleus-core/
git commit -m "feat(core): add crypto primitives, validation, and clock"
```

---

### Task 1.4: nucleus-db — Database & Redis Connection

**Files:**
- Create: `crates/nucleus-db/src/pool.rs`
- Create: `crates/nucleus-db/src/redis.rs`
- Modify: `crates/nucleus-db/src/lib.rs`

**Step 1: Implement PostgreSQL pool setup**

```rust
// pool.rs
use sqlx::postgres::{PgPool, PgPoolOptions};

pub async fn create_pg_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(200)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(600))
        .connect(database_url)
        .await
}
```

**Step 2: Implement Redis pool setup**

```rust
// redis.rs
use redis::aio::ConnectionManager;

pub async fn create_redis_pool(redis_url: &str) -> Result<ConnectionManager, redis::RedisError> {
    let client = redis::Client::open(redis_url)?;
    ConnectionManager::new(client).await
}
```

**Step 3: Verify compiles**

Run: `cargo check -p nucleus-db`
Expected: compiles

**Step 4: Commit**

```bash
git add crates/nucleus-db/
git commit -m "feat(db): add PostgreSQL and Redis connection pools"
```

---

### Task 1.5: nucleus-migrate — Database Migrations

**Files:**
- Create: `crates/nucleus-migrate/migrations/001_create_accounts.sql`
- Create: `crates/nucleus-migrate/migrations/002_create_plans.sql`
- Create: `crates/nucleus-migrate/migrations/003_create_projects.sql`
- Create: `crates/nucleus-migrate/migrations/004_create_users.sql`
- Create: `crates/nucleus-migrate/migrations/005_create_credentials.sql`
- Create: `crates/nucleus-migrate/migrations/006_create_mfa.sql`
- Create: `crates/nucleus-migrate/migrations/007_create_orgs.sql`
- Create: `crates/nucleus-migrate/migrations/008_create_roles_permissions.sql`
- Create: `crates/nucleus-migrate/migrations/009_create_audit_logs.sql`
- Create: `crates/nucleus-migrate/migrations/010_create_webhooks.sql`
- Create: `crates/nucleus-migrate/migrations/011_create_oauth_providers.sql`
- Create: `crates/nucleus-migrate/migrations/012_create_templates.sql`
- Create: `crates/nucleus-migrate/migrations/013_create_api_keys_signing_keys.sql`
- Modify: `crates/nucleus-migrate/src/lib.rs`

**Step 1: Write all 13 migration files**

Write complete SQL CREATE TABLE statements as specified in the design doc Section 4.

Key tables: accounts, plans, subscriptions, usage_metrics, projects, api_keys, signing_keys, jwt_templates, users, credentials, user_security, mfa_enrollments, verification_tokens, organizations, roles, permissions, role_permissions, org_members, invitations, oauth_providers, verified_domains, allowed_redirects, ip_rules, notification_templates, sign_in_attempts, audit_logs, webhook_events, webhook_delivery_logs.

Include all indexes, constraints, UNIQUE, and CHECK constraints.

**Step 2: Implement migration runner**

```rust
// lib.rs
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
```

**Step 3: Test migrations with testcontainers**

```rust
#[cfg(test)]
mod tests {
    use testcontainers::runners::AsyncRunner;
    use testcontainers_modules::postgres::Postgres;

    #[tokio::test]
    async fn migrations_apply_cleanly() {
        let pg = Postgres::default().start().await.unwrap();
        let url = format!("postgres://postgres:postgres@localhost:{}/postgres", pg.get_host_port_ipv4(5432).await.unwrap());
        let pool = PgPool::connect(&url).await.unwrap();
        run_migrations(&pool).await.unwrap();
        // Verify key tables exist
        let result = sqlx::query("SELECT tablename FROM pg_tables WHERE schemaname = 'public'")
            .fetch_all(&pool).await.unwrap();
        assert!(result.len() >= 20);
    }
}
```

**Step 4: Run migration test**

Run: `cargo test -p nucleus-migrate -- --nocapture`
Expected: PASS (requires Docker running)

**Step 5: Commit**

```bash
git add crates/nucleus-migrate/
git commit -m "feat(migrate): add all 13 database migration files"
```

---

### Task 1.6: nucleus-db — Repository Traits

**Files:**
- Create: `crates/nucleus-db/src/repos/mod.rs`
- Create: `crates/nucleus-db/src/repos/user_repo.rs`
- Create: `crates/nucleus-db/src/repos/project_repo.rs`
- Create: `crates/nucleus-db/src/repos/credential_repo.rs`
- Create: `crates/nucleus-db/src/repos/session_repo.rs`

**Step 1: Define repository traits**

```rust
// repos/user_repo.rs
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &NewUser) -> Result<User, AppError>;
    async fn find_by_id(&self, project_id: &ProjectId, user_id: &UserId) -> Result<Option<User>, AppError>;
    async fn find_by_email(&self, project_id: &ProjectId, email: &str) -> Result<Option<User>, AppError>;
    async fn update(&self, user: &UpdateUser) -> Result<User, AppError>;
    async fn soft_delete(&self, project_id: &ProjectId, user_id: &UserId) -> Result<(), AppError>;
    async fn list(&self, project_id: &ProjectId, params: &PaginationParams) -> Result<PaginatedResponse<User>, AppError>;
}
```

Define similar traits for: `ProjectRepository`, `CredentialRepository`, `SessionRepository` (Redis-backed), `OrgRepository`, `AuditLogRepository`, `WebhookEventRepository`, `ApiKeyRepository`.

**Step 2: Implement PostgreSQL-backed repos**

Implement `PgUserRepository`, `PgProjectRepository`, etc. using sqlx.

**Step 3: Implement Redis-backed session repo**

```rust
pub struct RedisSessionRepository {
    redis: ConnectionManager,
}

impl SessionRepository for RedisSessionRepository {
    async fn create(&self, session: &NewSession) -> Result<Session, AppError> {
        // HSET session:{id} fields...
        // SADD user_sessions:{user_id} session_id
        // EXPIRE session:{id} ttl
    }
    async fn find_by_id(&self, session_id: &SessionId) -> Result<Option<Session>, AppError> { ... }
    async fn delete(&self, session_id: &SessionId) -> Result<(), AppError> { ... }
    async fn delete_all_for_user(&self, user_id: &UserId) -> Result<(), AppError> { ... }
}
```

**Step 4: Test repos with testcontainers**

**Step 5: Commit**

```bash
git add crates/nucleus-db/
git commit -m "feat(db): add repository traits and PostgreSQL/Redis implementations"
```

---

### Task 1.7: nucleus-server — Config, Router, Middleware Skeleton

**Files:**
- Create: `crates/nucleus-server/src/config.rs`
- Create: `crates/nucleus-server/src/router.rs`
- Create: `crates/nucleus-server/src/state.rs`
- Create: `crates/nucleus-server/src/middleware/mod.rs`
- Create: `crates/nucleus-server/src/middleware/request_id.rs`
- Modify: `crates/nucleus-server/src/main.rs`

**Step 1: Implement config loading**

```rust
// config.rs
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub master_encryption_key: [u8; 32],
    pub host: String,
    pub port: u16,
    pub jwt_lifetime_secs: u64,
    pub session_ttl_secs: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        // read from env vars
    }
}
```

**Step 2: Implement AppState**

```rust
// state.rs
pub struct AppState {
    pub config: Config,
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub clock: Arc<dyn Clock>,
}
```

**Step 3: Implement main.rs with graceful shutdown**

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::init();
    let config = Config::from_env()?;
    let db = create_pg_pool(&config.database_url).await?;
    run_migrations(&db).await?;
    let redis = create_redis_pool(&config.redis_url).await?;
    let state = AppState { config, db, redis, clock: Arc::new(SystemClock) };
    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await?;
    Ok(())
}
```

**Step 4: Create router with health check**

```rust
// router.rs
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .with_state(Arc::new(state))
        .layer(TraceLayer::new_for_http())
        .layer(request_id_layer())
}
```

**Step 5: Verify server starts with docker-compose dev**

Create `deploy/docker-compose.dev.yml` with Postgres + Redis.

Run: `docker compose -f deploy/docker-compose.dev.yml up -d && cargo run -p nucleus-server`
Expected: Server starts, `/health` returns 200

**Step 6: Commit**

```bash
git add crates/nucleus-server/ deploy/
git commit -m "feat(server): add config, state, router, and health endpoint"
```

---

## Phase 2: Authentication — Password Sign-up/Sign-in, Sessions, JWT

### Task 2.1: nucleus-auth — Password Hashing Service

**Files:**
- Create: `crates/nucleus-auth/src/password.rs`
- Modify: `crates/nucleus-auth/src/lib.rs`

**Step 1: Write tests for password service**

Test: hash, verify, reject weak passwords, min/max length, rehash detection.

**Step 2: Implement password service**

Uses `nucleus-core::crypto::hash_password` and `verify_password`. Adds password policy validation (min 8, max 128, no arbitrary rules).

**Step 3: Run tests, commit**

---

### Task 2.2: nucleus-auth — JWT Signing & Verification

**Files:**
- Create: `crates/nucleus-auth/src/jwt.rs`

**Step 1: Write tests**

- sign + verify roundtrip (RS256)
- reject expired token
- reject tampered claims
- reject `alg: none`
- reject algorithm confusion (HS256 with RSA public key)
- kid matching
- claims template rendering

**Step 2: Implement JWT module**

- `sign_jwt(claims: &Claims, key: &SigningKey) -> Result<String>`
- `verify_jwt(token: &str, jwks: &Jwks) -> Result<Claims>`
- `render_claims_template(template: &JwtTemplate, user: &User, org: Option<&OrgMembership>) -> Claims`
- JWKS struct with key rotation support

**Step 3: Run tests, commit**

---

### Task 2.3: nucleus-session — Redis Session Management

**Files:**
- Create: `crates/nucleus-session/src/session.rs`
- Create: `crates/nucleus-session/src/refresh.rs`

**Step 1: Write tests (integration with testcontainers Redis)**

- create session → stored in Redis with TTL
- find session by ID
- refresh extends TTL + issues new JWT
- revoke session → delete from Redis
- revoke all sessions for user
- revoked session rejects refresh
- idle timeout enforcement

**Step 2: Implement session management**

Session create: generate 256-bit token, store HASH in Redis, add to user_sessions SET.
Session refresh: check session exists + not revoked, issue new JWT, update last_active.
Revocation list: `revoked_jwt:{jti}` with TTL = JWT lifetime.

**Step 3: Run tests, commit**

---

### Task 2.4: nucleus-auth — Sign-up Handler

**Files:**
- Create: `crates/nucleus-auth/src/handlers/mod.rs`
- Create: `crates/nucleus-auth/src/handlers/sign_up.rs`

**Step 1: Write integration test**

```rust
#[tokio::test]
async fn sign_up_creates_user_and_returns_session() {
    let app = test_app().await;
    let res = app.post("/api/v1/auth/sign-up")
        .json(&json!({ "email": "test@example.com", "password": "SecurePass123!" }))
        .send().await;
    assert_eq!(res.status(), 201);
    let body: Value = res.json().await;
    assert!(body["user"]["id"].is_string());
    assert!(body["jwt"].is_string());
    assert!(body["session_token"].is_string());
}

#[tokio::test]
async fn sign_up_rejects_duplicate_email() {
    // sign up once, then try again with same email
    // expect 409 with code "user/email_taken"
}

#[tokio::test]
async fn sign_up_rejects_weak_password() {
    // expect 422 with code "auth/password_too_weak"
}
```

**Step 2: Implement sign-up handler**

Flow: validate input → check email not taken → hash password → create user → create credential → create session in Redis → sign JWT → return response.

Anti-enumeration: always return 200 for sign-up even if email exists (send "email already registered" email instead). Actually, for sign-up it's acceptable to return 409 since the user is the one trying to register.

**Step 3: Run tests, commit**

---

### Task 2.5: nucleus-auth — Sign-in Handler

**Files:**
- Create: `crates/nucleus-auth/src/handlers/sign_in.rs`

**Step 1: Write integration tests**

- successful sign-in returns user + JWT + session_token
- wrong password returns 401 "auth/invalid_credentials" (same as wrong email)
- wrong email returns 401 "auth/invalid_credentials" (no enumeration)
- locked account returns 423
- banned account returns 403
- 5 failed attempts → account locks
- sign-in logs attempt to sign_in_attempts table

**Step 2: Implement sign-in handler**

Flow: find user by email → check not locked/banned → verify password → check MFA enrolled (if yes, return mfa_required) → create session → sign JWT → log sign-in attempt → return response.

**Step 3: Run tests, commit**

---

### Task 2.6: nucleus-auth — Token Refresh & Sign-out

**Files:**
- Create: `crates/nucleus-auth/src/handlers/token.rs`

**Step 1: Write tests**

- refresh with valid session → new JWT
- refresh with expired session → 401
- refresh with revoked session → 401
- sign-out → session deleted, subsequent refresh fails
- sign-out-all → all sessions deleted

**Step 2: Implement handlers**

- `POST /auth/token/refresh` — validate session in Redis, sign new JWT
- `POST /auth/sign-out` — delete session from Redis
- `POST /auth/sign-out/all` — delete all sessions from user_sessions SET

**Step 3: Run tests, commit**

---

### Task 2.7: nucleus-server — Auth Middleware

**Files:**
- Create: `crates/nucleus-server/src/middleware/auth.rs`

**Step 1: Write tests**

- request with valid JWT → extracts claims into request extensions
- request with expired JWT → 401
- request with invalid JWT → 401
- request with valid API key (secret) → extracts project context
- request with valid API key (publishable) → extracts project context
- request with revoked API key → 401
- check API key scopes

**Step 2: Implement auth middleware**

Two extractors:
- `JwtAuth` — extracts and verifies JWT from Authorization header
- `ApiKeyAuth` — extracts and verifies API key from Authorization header (for admin API)

**Step 3: Run tests, commit**

---

### Task 2.8: nucleus-server — Rate Limiting Middleware

**Files:**
- Create: `crates/nucleus-server/src/middleware/rate_limit.rs`

**Step 1: Write tests**

- under limit → request passes
- at limit → 429 with Retry-After header
- sliding window resets
- per-IP + per-project limits

**Step 2: Implement Redis-backed sliding window**

Uses Redis ZSET with timestamps. Remove entries older than window, count remaining, compare to limit.

**Step 3: Run tests, commit**

---

### Task 2.9: Wire Phase 2 Routes & Integration Test

**Files:**
- Modify: `crates/nucleus-server/src/router.rs`

**Step 1: Wire all Phase 2 routes**

```rust
Router::new()
    .route("/health", get(health))
    .nest("/api/v1/auth", auth_routes())
    // auth_routes:
    // POST /sign-up
    // POST /sign-in
    // POST /token/refresh
    // POST /sign-out
    // POST /sign-out/all
```

**Step 2: Full integration test — sign-up → sign-in → refresh → sign-out**

**Step 3: Commit**

```bash
git commit -m "feat: complete Phase 2 — password auth, sessions, JWT"
```

---

## Phase 3: Extended Authentication

### Task 3.1: OAuth (Google, GitHub, Apple, Microsoft)

**Files:**
- Create: `crates/nucleus-auth/src/oauth/mod.rs`
- Create: `crates/nucleus-auth/src/oauth/provider.rs`
- Create: `crates/nucleus-auth/src/oauth/google.rs`
- Create: `crates/nucleus-auth/src/oauth/github.rs`
- Create: `crates/nucleus-auth/src/oauth/apple.rs`
- Create: `crates/nucleus-auth/src/oauth/microsoft.rs`
- Create: `crates/nucleus-auth/src/pkce.rs`
- Create: `crates/nucleus-auth/src/handlers/oauth.rs`

Implement OAuthProvider trait, PKCE (S256), CSRF state management in Redis, callback handling, user creation/linking.

Tests: state mismatch rejection, PKCE verification, user creation on first OAuth sign-in, account linking on subsequent sign-in.

---

### Task 3.2: Magic Link

**Files:**
- Create: `crates/nucleus-auth/src/magic_link.rs`
- Create: `crates/nucleus-auth/src/handlers/magic_link.rs`

Generate token → store hash in verification_tokens → send email → verify token on click → create session.

Tests: token single-use, token expiry (15 min), rate limit (1/min), replay prevention.

---

### Task 3.3: OTP (Email/SMS)

**Files:**
- Create: `crates/nucleus-auth/src/otp.rs`
- Create: `crates/nucleus-auth/src/handlers/otp.rs`

Generate 6-digit code → store in Redis (5 min TTL, max 3 attempts) → verify.

Tests: correct code, wrong code, expired code, max attempts, rate limit.

---

### Task 3.4: MFA (TOTP + Backup Codes)

**Files:**
- Create: `crates/nucleus-auth/src/mfa.rs`
- Create: `crates/nucleus-auth/src/handlers/mfa.rs`

TOTP enrollment: generate secret → encrypt with AES-256-GCM → store → return QR URI.
TOTP verify: decrypt secret, validate code (±1 time step), reject replay.
Backup codes: generate 10x 8-char codes, encrypt, one-time use.

Tests: enrollment flow, verify correct/wrong code, ±1 drift, replay rejection, backup code single-use.

---

### Task 3.5: Passkeys (WebAuthn)

**Files:**
- Create: `crates/nucleus-auth/src/passkey.rs`
- Create: `crates/nucleus-auth/src/handlers/passkey.rs`

Uses `webauthn-rs` crate. Registration begin/finish, authentication begin/finish.

Tests: registration flow, authentication flow, origin validation, challenge expiry.

---

### Task 3.6: Password Reset

**Files:**
- Create: `crates/nucleus-auth/src/handlers/password_reset.rs`

Generate reset token → store hash → send email → verify token → update password → invalidate sessions.

Tests: token expiry (1 hour), single-use, rate limit (3/hour), no enumeration.

---

### Task 3.7: Wire Phase 3 Routes & Commit

Wire all new routes, run full test suite.

```bash
git commit -m "feat: complete Phase 3 — OAuth, magic link, OTP, MFA, passkeys, password reset"
```

---

## Phase 4: Identity & Organizations

### Task 4.1: nucleus-identity — User CRUD & Profile

**Files:**
- Create: `crates/nucleus-identity/src/user.rs`
- Create: `crates/nucleus-identity/src/user_security.rs`
- Create: `crates/nucleus-identity/src/credential.rs`
- Create: `crates/nucleus-identity/src/verification.rs`
- Create: `crates/nucleus-identity/src/handlers/me.rs`
- Create: `crates/nucleus-identity/src/handlers/admin.rs`

Public API: GET/PATCH/DELETE /users/me, GET /users/me/sessions
Admin API: GET/POST/PATCH/DELETE /admin/users, POST /admin/users/:id/ban|unban

Tests: CRUD operations, cross-tenant isolation, soft delete, metadata visibility (public vs private).

---

### Task 4.2: nucleus-org — Organizations CRUD

**Files:**
- Create: `crates/nucleus-org/src/organization.rs`
- Create: `crates/nucleus-org/src/member.rs`
- Create: `crates/nucleus-org/src/handlers/org.rs`
- Create: `crates/nucleus-org/src/handlers/member.rs`

Public API: CRUD orgs, list/add/remove members, change roles.
Tests: slug uniqueness, member limit enforcement, ownership transfer.

---

### Task 4.3: nucleus-org — RBAC (Roles & Permissions)

**Files:**
- Create: `crates/nucleus-org/src/role.rs`
- Create: `crates/nucleus-org/src/permission.rs`

System roles (owner, admin, member) + custom roles.
Permission resolution: user → org_member → role → role_permissions → permissions.

Tests: permission check for various role combinations, custom role creation, system role protection.

---

### Task 4.4: nucleus-org — Invitations

**Files:**
- Create: `crates/nucleus-org/src/invitation.rs`
- Create: `crates/nucleus-org/src/handlers/invitation.rs`

Invite flow: create invitation → send email → accept with token → add as member.

Tests: invitation expiry, already-accepted rejection, revocation.

---

### Task 4.5: Wire Phase 4 Routes & Commit

```bash
git commit -m "feat: complete Phase 4 — user management, organizations, RBAC, invitations"
```

---

## Phase 5: Webhooks & Admin API

### Task 5.1: nucleus-webhook — Event Dispatch & Delivery

**Files:**
- Create: `crates/nucleus-webhook/src/events.rs`
- Create: `crates/nucleus-webhook/src/dispatcher.rs`
- Create: `crates/nucleus-webhook/src/delivery.rs`
- Create: `crates/nucleus-webhook/src/handlers/admin.rs`

Events: user.created, user.updated, user.deleted, session.created, session.revoked, org.created, org.member.added, org.member.removed, etc.

Outbox pattern: write to webhook_events table → background task picks up → HTTP POST with HMAC-SHA256 signature → retry with exponential backoff (5 attempts).

Tests: HMAC signing, delivery success/failure, retry logic, replay protection (timestamp check), SSRF protection on webhook URL.

---

### Task 5.2: nucleus-admin-api — Dashboard API

**Files:**
- Create: `crates/nucleus-admin-api/src/project.rs`
- Create: `crates/nucleus-admin-api/src/billing.rs`
- Create: `crates/nucleus-admin-api/src/analytics.rs`
- Create: `crates/nucleus-admin-api/src/settings.rs`
- Create: `crates/nucleus-admin-api/src/provider.rs`
- Create: `crates/nucleus-admin-api/src/template.rs`
- Create: `crates/nucleus-admin-api/src/api_key.rs`
- Create: `crates/nucleus-admin-api/src/signing_key.rs`
- Create: `crates/nucleus-admin-api/src/handlers/dashboard.rs`

Project CRUD, OAuth provider config, email/SMS template management, API key generation/revocation, signing key rotation, audit log queries, analytics (sign-in stats, MAU).

---

### Task 5.3: JWKS & OpenID Discovery Endpoints

**Files:**
- Modify: `crates/nucleus-server/src/router.rs`

`GET /.well-known/jwks.json` — public signing keys
`GET /.well-known/openid-configuration` — OpenID Connect discovery

Tests: JWKS returns valid key format, key rotation reflected, cache headers set.

---

### Task 5.4: Federated Mode Template & API

**Files:**
- Create: `templates/nucleus_template.sql`

Generate the SQL template file that federated-mode projects import. Webhook events trigger sync.

---

### Task 5.5: Wire Phase 5 & Full Integration Test

```bash
git commit -m "feat: complete Phase 5 — webhooks, admin API, JWKS, federated mode"
```

---

## Phase 6: Admin Dashboard (React SPA)

### Task 6.1: Scaffold Dashboard

**Files:**
- Create: `dashboard/package.json`
- Create: `dashboard/vite.config.ts`
- Create: `dashboard/tsconfig.json`
- Create: `dashboard/src/main.tsx`
- Create: `dashboard/src/App.tsx`
- Create: `dashboard/src/lib/api.ts`

Scaffold with Vite + React + TypeScript + TailwindCSS. API client for dashboard endpoints.

---

### Task 6.2: Auth Pages (Login, Register)

Dashboard account login/register using Nucleus's own auth endpoints.

---

### Task 6.3: Project Management Pages

Pages: project list, project create, project overview.

---

### Task 6.4: User & Org Management Pages

Pages: user list, user detail, org list, org detail, member management.

---

### Task 6.5: Settings Pages

Pages: auth settings, OAuth providers, email templates, API keys, JWT templates, webhook config, audit log viewer, analytics dashboard, billing.

---

### Task 6.6: Commit Dashboard

```bash
git commit -m "feat: complete Phase 6 — admin dashboard SPA"
```

---

## Phase 7: Phase 1 SDKs (MVP)

### Task 7.1: @nucleus/node — Backend Node.js SDK

**Files:**
- Create: `sdks/node/package.json`
- Create: `sdks/node/tsconfig.json`
- Create: `sdks/node/tsup.config.ts`
- Create: `sdks/node/src/index.ts`
- Create: `sdks/node/src/client.ts`
- Create: `sdks/node/src/verify.ts`
- Create: `sdks/node/src/admin/users.ts`
- Create: `sdks/node/src/admin/orgs.ts`
- Create: `sdks/node/src/express.ts`
- Create: `sdks/node/src/fastify.ts`
- Create: `sdks/node/src/hono.ts`
- Create: `sdks/node/src/types.ts`

Layer 1 (verify) + Layer 2 (admin API) + framework middleware (Express, Fastify, Hono).

Package config: dual ESM + CJS, exports field, compatible with npm/yarn/pnpm/bun.

---

### Task 7.2: @nucleus/nextjs — Frontend Next.js SDK

**Files:**
- Create: `sdks/nextjs/package.json`
- Create: `sdks/nextjs/src/index.ts`
- Create: `sdks/nextjs/src/provider.tsx`
- Create: `sdks/nextjs/src/hooks/use-user.ts`
- Create: `sdks/nextjs/src/hooks/use-session.ts`
- Create: `sdks/nextjs/src/hooks/use-auth.ts`
- Create: `sdks/nextjs/src/hooks/use-sign-in.ts`
- Create: `sdks/nextjs/src/hooks/use-sign-up.ts`
- Create: `sdks/nextjs/src/hooks/use-organization.ts`
- Create: `sdks/nextjs/src/hooks/use-organization-list.ts`
- Create: `sdks/nextjs/src/components/sign-in.tsx`
- Create: `sdks/nextjs/src/components/sign-up.tsx`
- Create: `sdks/nextjs/src/components/user-button.tsx`
- Create: `sdks/nextjs/src/components/user-profile.tsx`
- Create: `sdks/nextjs/src/components/org-switcher.tsx`
- Create: `sdks/nextjs/src/components/org-profile.tsx`
- Create: `sdks/nextjs/src/server/auth.ts`
- Create: `sdks/nextjs/src/server/middleware.ts`
- Create: `sdks/nextjs/src/server/token.ts`
- Create: `sdks/nextjs/src/client/api.ts`
- Create: `sdks/nextjs/src/client/session.ts`
- Create: `sdks/nextjs/src/client/types.ts`

All 3 layers: verify + admin + UI components. NucleusProvider, hooks, pre-built components, Edge middleware.

---

### Task 7.3: nucleus_flutter — Mobile Flutter SDK

**Files:**
- Create: `sdks/flutter/pubspec.yaml`
- Create: `sdks/flutter/lib/nucleus.dart`
- Create: `sdks/flutter/lib/src/config.dart`
- Create: `sdks/flutter/lib/src/client.dart`
- Create: `sdks/flutter/lib/src/auth/auth_state.dart`
- Create: `sdks/flutter/lib/src/auth/sign_in.dart`
- Create: `sdks/flutter/lib/src/auth/sign_up.dart`
- Create: `sdks/flutter/lib/src/auth/oauth.dart`
- Create: `sdks/flutter/lib/src/auth/mfa.dart`
- Create: `sdks/flutter/lib/src/session/session_manager.dart`
- Create: `sdks/flutter/lib/src/session/token_storage.dart`
- Create: `sdks/flutter/lib/src/session/auto_refresh.dart`
- Create: `sdks/flutter/lib/src/models/user.dart`
- Create: `sdks/flutter/lib/src/models/session.dart`
- Create: `sdks/flutter/lib/src/models/organization.dart`
- Create: `sdks/flutter/lib/src/widgets/nucleus_provider.dart`
- Create: `sdks/flutter/lib/src/widgets/sign_in_widget.dart`
- Create: `sdks/flutter/lib/src/widgets/sign_up_widget.dart`
- Create: `sdks/flutter/lib/src/widgets/user_button.dart`
- Create: `sdks/flutter/lib/src/widgets/org_switcher.dart`

PKCE OAuth, deep link handling, Keychain/EncryptedSharedPreferences, auto-refresh.

---

### Task 7.4: nucleus-java — Backend Java SDK (Spring Boot)

**Files:**
- Create: `sdks/java/pom.xml`
- Create: `sdks/java/src/main/java/dev/nucleus/NucleusClient.java`
- Create: `sdks/java/src/main/java/dev/nucleus/NucleusClaims.java`
- Create: `sdks/java/src/main/java/dev/nucleus/admin/UsersApi.java`
- Create: `sdks/java/src/main/java/dev/nucleus/admin/OrgsApi.java`
- Create: `sdks/java/src/main/java/dev/nucleus/spring/NucleusAutoConfiguration.java`
- Create: `sdks/java/src/main/java/dev/nucleus/spring/NucleusAuthFilter.java`
- Create: `sdks/java/src/main/java/dev/nucleus/spring/NucleusPermission.java`
- Create: `sdks/java/src/main/java/dev/nucleus/spring/NucleusProperties.java`

JWKS caching, JWT verification, Spring Security filter, `@NucleusPermission` annotation, admin API client.

---

### Task 7.5: nucleus-py — Backend Python SDK

**Files:**
- Create: `sdks/python/pyproject.toml`
- Create: `sdks/python/src/nucleus/__init__.py`
- Create: `sdks/python/src/nucleus/client.py`
- Create: `sdks/python/src/nucleus/verify.py`
- Create: `sdks/python/src/nucleus/claims.py`
- Create: `sdks/python/src/nucleus/admin/users.py`
- Create: `sdks/python/src/nucleus/admin/orgs.py`
- Create: `sdks/python/src/nucleus/fastapi.py`
- Create: `sdks/python/src/nucleus/django.py`
- Create: `sdks/python/src/nucleus/flask.py`
- Create: `sdks/python/src/nucleus/sync.py`

Async + sync clients, FastAPI Depends, Django middleware, Flask decorator.

---

### Task 7.6: Commit Phase 1 SDKs

```bash
git commit -m "feat: complete Phase 7 — Phase 1 SDKs (Next.js, Flutter, Java, Node, Python)"
```

---

## Phase 8: Phase 2 SDKs (Growth)

### Task 8.1: nucleus-go — Backend Go SDK

Go module with JWKS caching, JWT verify, admin API, middleware for net/http, Gin, Fiber.

### Task 8.2: NucleusSwift — iOS Native SDK

SPM package with SwiftUI views, Keychain storage, ASWebAuthenticationSession (OAuth), ASAuthorizationPlatformPublicKeyCredentialProvider (Passkeys), deep link handling.

### Task 8.3: nucleus-android — Android Kotlin SDK

Maven Central package with Jetpack Compose components, EncryptedSharedPreferences, Credential Manager API (Passkeys), deep link handling, Retrofit/OkHttp interceptor.

### Task 8.4: @nucleus/react — React SPA SDK

React hooks + components (no Next.js-specific features like middleware).

### Task 8.5: Commit Phase 2 SDKs

```bash
git commit -m "feat: complete Phase 8 — Phase 2 SDKs (Go, Swift, Android, React)"
```

---

## Phase 9: Phase 3 SDKs (Enterprise)

### Task 9.1: Nucleus.NET — C# SDK

NuGet package with ASP.NET Core DI, authentication middleware, admin API client.

### Task 9.2: nucleus-rs — Rust SDK

crates.io package with JWKS caching, JWT verify, admin API, Axum/Actix-web middleware.

### Task 9.3: nucleus-android-java — Android Java SDK

Thin wrapper over Kotlin SDK with Java-idiomatic API (callbacks, LiveData, XML layout components).

### Task 9.4: @nucleus/js — Vanilla JS SDK

Web Components for sign-in/sign-up, framework-agnostic.

### Task 9.5: Commit Phase 3 SDKs

```bash
git commit -m "feat: complete Phase 9 — Phase 3 SDKs (.NET, Rust, Android Java, Vanilla JS)"
```

---

## Phase 10: Deployment & CI/CD

### Task 10.1: Docker & Docker Compose

**Files:**
- Create: `deploy/Dockerfile`
- Create: `deploy/docker-compose.yml`
- Create: `deploy/docker-compose.dev.yml`

Multi-stage Rust build → Alpine runtime (~30MB image).

---

### Task 10.2: Kubernetes Manifests

**Files:**
- Create: `deploy/k8s/deployment.yaml`
- Create: `deploy/k8s/service.yaml`
- Create: `deploy/k8s/configmap.yaml`
- Create: `deploy/k8s/ingress.yaml`

---

### Task 10.3: CI — Tests + Coverage

**Files:**
- Create: `.github/workflows/ci.yml`

cargo-nextest + cargo-llvm-cov + Codecov + Allure report + benchmark-action.

---

### Task 10.4: CI — Security Pipeline

**Files:**
- Create: `.github/workflows/security.yml`

Snyk SCA + Snyk Code + cargo-audit + cargo-deny.

---

### Task 10.5: CI — Claude Analysis

**Files:**
- Create: `.github/workflows/claude-analysis.yml`
- Create: `.github/workflows/weekly-digest.yml`

Claude PR analysis + weekly engineering digest.

---

### Task 10.6: CI — Release Pipeline

**Files:**
- Create: `.github/workflows/release.yml`

Build binary, Docker image, SBOM, publish to registries.

---

### Task 10.7: Commit Phase 10

```bash
git commit -m "feat: complete Phase 10 — Docker, K8s, CI/CD pipelines"
```

---

## Phase 11: Load Testing, Monitoring & Observability

### Task 11.1: Prometheus Metrics

**Files:**
- Modify: `crates/nucleus-server/src/middleware/mod.rs`

Add metrics endpoint `/metrics` with counters/histograms from design:
- nucleus_auth_attempts_total
- nucleus_auth_latency_seconds
- nucleus_sessions_active
- nucleus_rate_limit_hits_total
- nucleus_jwt_issued_total
- nucleus_webhook_deliveries_total

---

### Task 11.2: k6 Load Tests

**Files:**
- Create: `load-tests/auth-flow.k6.js`
- Create: `load-tests/token-refresh.k6.js`
- Create: `load-tests/admin-api.k6.js`
- Create: `.github/workflows/load-test.yml`

Scenarios: steady state (100 rps), spike (500 rps), soak (30 min).
Thresholds from design doc performance budgets.

---

### Task 11.3: Criterion Benchmarks

**Files:**
- Create: `benches/auth_benchmarks.rs`
- Create: `benches/session_benchmarks.rs`
- Create: `benches/jwt_benchmarks.rs`

Benchmark: argon2id hash, JWT sign/verify, session create, token refresh.

---

### Task 11.4: Grafana Dashboard Configs

**Files:**
- Create: `deploy/grafana/nucleus-dashboard.json`
- Create: `deploy/grafana/k6-dashboard.json`

Pre-built Grafana dashboards for production metrics and load test results.

---

### Task 11.5: Commit Phase 11

```bash
git commit -m "feat: complete Phase 11 — load tests, benchmarks, monitoring"
```

---

## Summary

| Phase | Description | Estimated Tasks |
|:------|:------------|:----------------|
| 1 | Foundation (workspace, core, DB, migrations) | 7 tasks |
| 2 | Auth basics (password, sessions, JWT, middleware) | 9 tasks |
| 3 | Extended auth (OAuth, magic link, OTP, MFA, passkeys) | 7 tasks |
| 4 | Identity & Organizations (CRUD, RBAC, invitations) | 5 tasks |
| 5 | Webhooks & Admin API (events, dashboard API, JWKS) | 5 tasks |
| 6 | Dashboard SPA (React + Vite) | 6 tasks |
| 7 | Phase 1 SDKs (Next.js, Flutter, Java, Node, Python) | 6 tasks |
| 8 | Phase 2 SDKs (Go, Swift, Android, React) | 5 tasks |
| 9 | Phase 3 SDKs (.NET, Rust, Android Java, Vanilla JS) | 5 tasks |
| 10 | Deployment & CI/CD (Docker, K8s, pipelines) | 7 tasks |
| 11 | Load testing & Monitoring (k6, benchmarks, Grafana) | 5 tasks |
| **Total** | | **67 tasks** |

### Execution Order

Phases 1-5 are **strictly sequential** (each depends on the previous).

After Phase 5, these can run **in parallel**:
- Phase 6 (Dashboard) — depends on Phase 5
- Phase 7-9 (SDKs) — depends on Phase 5
- Phase 10-11 (CI/DevOps) — can start after Phase 2

### Critical Path

```
Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5 → Phase 7 (MVP SDKs)
                                                  ↘ Phase 6 (Dashboard)
         Phase 2 → Phase 10 (CI/CD, can start early)
         Phase 5 → Phase 11 (Load tests)
```

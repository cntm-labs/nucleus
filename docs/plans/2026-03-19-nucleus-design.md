# Nucleus — Authentication SaaS Platform Design

**Date:** 2026-03-19
**Status:** Approved
**Author:** mrbt + Claude

---

## 1. Overview

Nucleus is an authentication-as-a-service platform (similar to Clerk.com) that provides:

- **Backend API** — Rust-based auth server handling sign-up, sign-in, sessions, MFA, OAuth, Passkeys, organizations, RBAC, webhooks, and API keys
- **Frontend SDKs** — Pre-built UI components and hooks for Next.js, React, and vanilla JS
- **Mobile SDKs** — Native SDKs for Flutter, Swift (iOS), and Android (Kotlin/Java)
- **Backend SDKs** — Token verification + Admin API clients for Rust, Go, C#, Java, Python, and Node.js
- **Admin Dashboard** — React SPA for managing projects, users, organizations, and settings

### Key Decisions

| Decision | Choice | Rationale |
|:---------|:-------|:----------|
| Deployment | Hybrid (cloud + self-hosted) | Maximum market reach |
| Multi-tenancy | Shared DB, shared schema (`tenant_id`) | Simple, cost-effective, easy to start |
| Session strategy | Hybrid JWT + Redis | Short-lived JWT (5 min) for speed + Redis sessions for revocation |
| Architecture | Modular Monolith (Rust) | Single binary for self-hosted, easy to develop/debug |
| Backend language | Rust (Axum) | Performance, memory safety, single binary deployment |
| Dashboard | React SPA (Vite) | Separate from backend, communicates via API |
| Data sovereignty | Centralized + Federated modes | Projects choose where user data lives |

---

## 2. Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Nucleus Platform                       │
│                                                           │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────┐  │
│  │ Admin SPA   │  │ Frontend SDKs│  │  Mobile SDKs   │  │
│  │ (React)     │  │ (Next.js etc)│  │(Flutter/Swift/ │  │
│  └──────┬──────┘  └──────┬───────┘  │ Android)       │  │
│         │                │           └───────┬────────┘  │
│         └────────────────┼───────────────────┘           │
│                          ▼                               │
│  ┌───────────────────────────────────────────────────┐   │
│  │            Nucleus Server (Rust binary)            │   │
│  │                                                    │   │
│  │  ┌────────┐ ┌──────────┐ ┌─────┐ ┌───────────┐   │   │
│  │  │  Auth  │ │ Identity │ │ Org │ │  Webhook  │   │   │
│  │  └────┬───┘ └─────┬────┘ └──┬──┘ └─────┬─────┘   │   │
│  │       └───────────┼─────────┼───────────┘         │   │
│  │                   ▼         ▼                     │   │
│  │              ┌─────────┐ ┌───────┐                │   │
│  │              │  Core   │ │Session│                │   │
│  │              └────┬────┘ └───┬───┘                │   │
│  └───────────────────┼─────────┼─────────────────────┘   │
│                      ▼         ▼                         │
│               ┌──────────┐ ┌───────┐                     │
│               │PostgreSQL│ │ Redis │                     │
│               └──────────┘ └───────┘                     │
└─────────────────────────────────────────────────────────┘
```

### Tech Stack

| Component | Technology |
|:----------|:----------|
| Backend API | Rust (Axum framework) |
| Database | PostgreSQL 16 |
| Cache/Sessions | Redis 7 |
| Admin Dashboard | React SPA (Vite) |
| Frontend SDK | TypeScript (Next.js, React, vanilla JS) |
| Mobile SDKs | Dart (Flutter), Swift (iOS), Kotlin/Java (Android) |
| Backend SDKs | Rust, Go, C#, Java, Python, Node.js |
| API Protocol | REST (JSON) |
| Package managers | npm, yarn, pnpm, bun (JS/TS SDKs) |

---

## 3. Data Sovereignty Strategy

### Mode A: Centralized (user data in Nucleus)

External projects call Nucleus Admin API for user data. Endpoints:
- `GET /api/v1/admin/users/:id`
- `GET /api/v1/admin/users?email=...`
- `PATCH /api/v1/admin/users/:id/metadata`

### Mode B: Federated (user data in project's own DB)

- Nucleus provides database migration template SQL
- Nucleus stores only auth data (credentials, sessions, MFA)
- User profile data synced via webhook events (`user.created`, `user.updated`, `user.deleted`)
- JWT claims contain `nucleus_user_id` for mapping

### Rich JWT Claims (zero network calls for backends)

Configurable per project via JWT Templates:
```json
{
  "sub": "user_abc123",
  "email": "john@example.com",
  "first_name": "John",
  "org_id": "org_xyz",
  "org_role": "admin",
  "org_permissions": ["billing:read", "members:invite"],
  "metadata": { "plan": "pro" }
}
```

Backends verify JWT locally via JWKS (`/.well-known/jwks.json`) — zero calls to Nucleus per request.

---

## 4. Database Schema

### Core Tables

**Platform layer:**
- `accounts` — Nucleus platform accounts
- `plans` — Subscription plans (free, pro, enterprise)
- `subscriptions` — Active subscriptions per project
- `usage_metrics` — MAU, API requests tracking

**Project/Tenant layer:**
- `projects` — Each project using Nucleus (data_mode, webhook_url, session_ttl, jwt_lifetime, settings)
- `api_keys` — Publishable + secret keys with scopes
- `signing_keys` — JWT signing keys with rotation support
- `jwt_templates` — Custom claims templates per project

**Identity layer:**
- `users` — User profiles (per project, with metadata + private_metadata)
- `credentials` — Multi-method auth (password, oauth, magic_link, passkey, otp)
- `user_security` — Failed attempts, lockout, ban state
- `mfa_enrollments` — TOTP, SMS, email, backup codes
- `verification_tokens` — Email verify, password reset, magic link tokens

**Organization layer:**
- `organizations` — Orgs per project
- `roles` — RBAC roles (system + custom)
- `permissions` — Permission keys per project
- `role_permissions` — Role-to-permission mapping
- `org_members` — User-org membership with role
- `invitations` — Invite flow with token + expiry

**Auth configuration:**
- `oauth_providers` — OAuth config per project (Google, GitHub, Apple, etc.)
- `verified_domains` — Enterprise SSO domain verification
- `allowed_redirects` — Security: allowed redirect URIs
- `ip_rules` — IP allowlist/blocklist per project
- `notification_templates` — Email/SMS templates (verification, magic link, etc.)

**Audit & observability:**
- `sign_in_attempts` — Every sign-in attempt with IP, geo, status
- `audit_logs` — Full compliance-grade audit trail
- `webhook_events` — Outbox pattern for webhook delivery
- `webhook_delivery_logs` — Delivery attempt logs

### Redis Data Structures

- `session:{id}` — Active sessions (HASH, TTL per project)
- `revoked_jwt:{jti}` — JWT revocation list (TTL = JWT lifetime)
- `rate:{project}:{ip}:{endpoint}` — Sliding window rate limiting
- `otp:{project}:{user}:{purpose}` — OTP codes (5 min TTL)
- `oauth_state:{state}` — CSRF protection (10 min TTL)
- `magic_link:{nonce}` — Replay prevention (15 min TTL)
- `user_sessions:{user_id}` — SET of session IDs (for sign-out-all)
- `usage:{project}:{YYYY-MM}` — Real-time usage counters

---

## 5. API Design

### API Structure

```
/api/v1/              ← Public API (Publishable Key)
  /auth/              ← Sign-in, Sign-up, OAuth, MFA, Passkeys
  /sessions/          ← Session management
  /users/me           ← Current user profile
  /orgs/              ← Organization (user-facing)

/api/v1/admin/        ← Server API (Secret Key, backend-to-backend)
  /users/             ← CRUD users
  /orgs/              ← CRUD organizations
  /invitations/       ← Manage invitations
  /audit-logs/        ← Query audit logs
  /webhooks/          ← Webhook management

/api/v1/dashboard/    ← Dashboard API (account session)
  /projects/          ← Manage projects
  /billing/           ← Plans, subscriptions, usage
  /providers/         ← OAuth config
  /templates/         ← Email/SMS templates
  /analytics/         ← Sign-in stats, MAU

/.well-known/         ← Discovery
  /jwks.json          ← Public signing keys
  /openid-configuration
```

### Auth Endpoints (Public)

- `POST /auth/sign-up` — Email/password registration
- `POST /auth/sign-in` — Email/password login
- `POST /auth/sign-in/oauth` — OAuth initiation
- `GET /auth/oauth/callback` — OAuth callback
- `POST /auth/sign-in/magic-link` — Send magic link
- `POST /auth/sign-in/otp/send` — Send OTP
- `POST /auth/sign-in/otp/verify` — Verify OTP
- `POST /auth/mfa/verify` — MFA verification
- `POST /auth/passkey/register/begin|finish` — Passkey registration
- `POST /auth/passkey/authenticate/begin|finish` — Passkey auth
- `POST /auth/token/refresh` — JWT refresh (checks Redis session)
- `POST /auth/sign-out` — Revoke current session
- `POST /auth/sign-out/all` — Revoke all sessions
- `POST /auth/password/reset` — Send reset link
- `POST /auth/password/reset/confirm` — Complete reset

### Hybrid Session Flow

1. User login → server creates session in Redis + issues JWT (5 min lifetime)
2. SDK uses JWT for all requests → backend verifies signature only (no Redis)
3. JWT expires → SDK uses session token to refresh (Redis check: session active?)
4. Revoke session → delete from Redis → next refresh fails → JWT expires in ≤5 min

---

## 6. Error Handling

### Error Response Format

```json
{
  "error": {
    "code": "auth/invalid_credentials",
    "message": "The email or password you entered is incorrect.",
    "status": 401,
    "request_id": "req_abc123xyz",
    "details": [{ "field": "password", "issue": "invalid" }],
    "docs_url": "https://docs.nucleus.dev/errors/auth-invalid-credentials"
  }
}
```

### Error Code Categories

- `auth/*` — Authentication errors (invalid_credentials, account_locked, mfa_required, session_expired, etc.)
- `user/*` — User errors (not_found, email_taken, etc.)
- `org/*` — Organization errors (slug_taken, member_limit_reached, insufficient_permissions, etc.)
- `api/*` — API-level errors (invalid_api_key, rate_limited, plan_limit_exceeded, etc.)

---

## 7. Security

### OWASP Compliance

Full OWASP Top 10 (2021) coverage and OWASP ASVS Level 2 compliance for:
- V2: Authentication (password security, MFA, passkeys, credential recovery)
- V3: Session management (regeneration, timeouts, concurrent limits)
- V4: Access control (deny by default, RBAC, scope enforcement)

### Cryptography Stack

| Purpose | Algorithm | Rust Crate |
|:--------|:----------|:-----------|
| Password hashing | Argon2id (19MB memory, 2 iterations) | argon2 |
| JWT signing | RS256 (RSA 2048) / ES256 (P-256) | jsonwebtoken + ring |
| Encryption at rest | AES-256-GCM (envelope encryption) | aes-gcm |
| Token generation | 256-bit CSPRNG | ring |
| Webhook signing | HMAC-SHA256 | hmac + sha2 |
| API key storage | SHA-256 hash | sha2 |
| TOTP | HMAC-SHA1 (RFC 6238) | ring |
| Passkeys | WebAuthn/FIDO2 | webauthn-rs |
| TLS | 1.2+ (prefer 1.3), AEAD ciphers only | rustls |

### Key Security Features

- Anti-enumeration: generic "invalid credentials" errors
- Brute force protection: progressive lockout (5 → lock 15 min, 10 → lock 1 hour)
- Breached password check: HaveIBeenPwned API
- Constant-time comparisons for all secrets
- PKCE for OAuth (mobile + web)
- Webhook replay protection (timestamp + HMAC)
- SSRF protection on webhook URLs (deny private IPs)
- PII scrubbing in logs
- Soft delete + 30-day hard purge (GDPR)
- Signing key rotation (90-day default)
- Envelope encryption (DEK encrypted with MEK from KMS)

### Snyk Integration (CI)

1. **Snyk SCA** — Dependency vulnerability scanning (Cargo.lock)
2. **Snyk Code** — Static analysis (SAST) for Rust source
3. **Snyk Container** — Docker image scanning
4. **Snyk IaC** — Infrastructure config scanning
5. **cargo-audit** — RustSec Advisory DB
6. **cargo-deny** — License + banned crate checks

---

## 8. Testing & Visualization

### Test Pyramid

- **Unit tests (~60%)** — Business logic, crypto, validation (Rust `#[test]`)
- **Integration tests (~30%)** — Full auth flows with real Postgres + Redis (testcontainers)
- **E2E tests (~10%)** — SDK flows (Playwright, Flutter integration tests)

### Security Test Categories

- T1: Cryptography (argon2id, JWT algorithms, AES-GCM, HMAC, constant-time)
- T2: Authentication (enumeration, brute force, MFA bypass, passkey origin)
- T3: Session (regeneration, timeout, revocation, fixation)
- T4: Access Control (cross-tenant isolation, API key scopes, RBAC)
- T5: Injection (SQL, XSS, SSRF, open redirect, unicode normalization)
- T6: Rate Limiting (per-IP, per-project, per-endpoint)
- T7: Data Protection (PII in logs, soft delete, metadata visibility)

### Visualization Tools

| Tool | Purpose |
|:-----|:--------|
| cargo-nextest | Fast test runner + JUnit XML |
| Allure Report | Interactive HTML test reports |
| cargo-llvm-cov + Codecov | Coverage trends + PR annotations |
| criterion.rs | Statistical benchmarks + HTML reports |
| k6 + Grafana | Load testing with real-time dashboards |
| cargo-flamegraph | CPU profiling (SVG flamegraphs) |
| DHAT / bytehound | Memory profiling + leak detection |
| Prometheus + Grafana | Runtime metrics dashboards |
| Snyk Dashboard | Vulnerability tracking over time |

### Performance Budgets (CI gates)

| Endpoint | P95 Budget |
|:---------|:-----------|
| POST /auth/sign-in | < 200ms |
| POST /auth/sign-up | < 250ms |
| POST /token/refresh | < 50ms |
| GET /users/me | < 30ms |
| GET /.well-known/jwks | < 10ms |

Additional: binary < 50MB, startup < 3s, idle memory < 100MB, memory growth < 1%/hour.

### AI-Powered CI (Claude in GitHub Actions)

- **Per-PR analysis**: Claude reads test results + coverage + benchmarks + code diff → generates structured PR comment with critical issues, warnings, positive changes, performance report, security report, and suggestions
- **Weekly digest**: Trends, action items, wins, predictions, recommended focus areas → posted to Slack + GitHub Discussions
- **Pre-push hook**: Local Claude analysis for security, missing tests, performance flags, API contract breaks
- Uses `claude-sonnet-4-6` for per-PR (fast), `claude-opus-4-6` for weekly digest (deep analysis)

---

## 9. Rust Crate Structure

```
crates/
├── nucleus-server       ← binary (main, config, router, middleware)
├── nucleus-core         ← shared types, errors, crypto, validation
├── nucleus-auth         ← sign-in, sign-up, OAuth, MFA, passkeys, JWT
├── nucleus-identity     ← user CRUD, security, credentials
├── nucleus-org          ← organizations, RBAC, invitations, domains
├── nucleus-session      ← Redis sessions, refresh, device tracking
├── nucleus-webhook      ← event dispatch, HMAC signing, retry delivery
├── nucleus-admin-api    ← dashboard API, billing, analytics, settings
├── nucleus-db           ← sqlx pool, Redis pool, repository pattern
└── nucleus-migrate      ← SQL migration files (001-013)
```

### Dependency Rules (enforced by cargo-deny)

- `nucleus-core` → no other nucleus crates (leaf)
- `nucleus-db` → only nucleus-core
- Feature crates (auth, identity, org, etc.) → nucleus-core + nucleus-db only
- No cross-feature dependencies (auth ↛ org, identity ↛ auth)
- `nucleus-server` → all crates (composition root)

---

## 10. SDK Ecosystem

### 14 SDKs across 5 platforms

**Frontend SDKs (npm — supports npm, yarn, pnpm, bun):**
- `@nucleus/nextjs` — Next.js (Provider, hooks, components, middleware)
- `@nucleus/react` — React SPA
- `@nucleus/js` — Vanilla JS / Web Components

**Mobile SDKs:**
- `nucleus_flutter` (pub.dev) — Flutter widgets + secure storage
- `NucleusSwift` (SPM/CocoaPods) — SwiftUI views + Keychain + ASWebAuthenticationSession + Passkeys
- `nucleus-android` (Maven Central) — Kotlin + Jetpack Compose + EncryptedSharedPreferences
- `nucleus-android-java` (Maven Central) — Java + XML layouts (thin wrapper over Kotlin SDK)

**Backend SDKs:**
- `nucleus-rs` (crates.io) — Rust + Axum/Actix-web middleware
- `nucleus-go` (go module) — Go + net/http/Gin/Fiber middleware
- `Nucleus.NET` (NuGet) — C# + ASP.NET Core DI + middleware
- `nucleus-java` (Maven Central) — Java + Spring Boot auto-config + `@NucleusPermission` annotation
- `nucleus-py` (PyPI) — Python + FastAPI/Django/Flask (async + sync clients)
- `@nucleus/node` (npm) — Node.js + Express/Fastify/Hono middleware

### SDK Layers

All SDKs implement:
1. **Token verification** — JWKS fetch + cache, local JWT verify, typed claims
2. **Admin API client** — CRUD users/orgs/sessions (backend SDKs only with secret key)
3. **UI components** — Pre-built sign-in/sign-up/profile/org-switcher (frontend/mobile SDKs only)

### JS/TS Package Compatibility

All JS/TS packages ship dual ESM + CJS via tsup, with proper `exports` field in package.json for compatibility across npm, yarn, pnpm, and bun.

### Release Phases

- **Phase 1 (MVP):** @nucleus/nextjs, nucleus_flutter, nucleus-java, @nucleus/node, nucleus-py
- **Phase 2 (growth):** nucleus-go, NucleusSwift, nucleus-android, @nucleus/react
- **Phase 3 (enterprise):** Nucleus.NET, nucleus-rs, nucleus-android-java, @nucleus/js

---

## 11. Deployment

### Self-hosted (single command)

```bash
docker compose up -d
# → Nucleus server + PostgreSQL + Redis
# → Ready at http://localhost:3000
```

### Docker Image

- Multi-stage build: Rust builder → Alpine runtime
- Image size: ~30MB (musl + stripped binary)
- Health check endpoint: `/health`

### Infrastructure

- `deploy/Dockerfile` — Multi-stage Rust build
- `deploy/docker-compose.yml` — Production (server + postgres + redis)
- `deploy/docker-compose.dev.yml` — Development with hot-reload
- `deploy/k8s/` — Kubernetes manifests (deployment, service, configmap, ingress)

---

## 12. CI/CD Pipelines

| Workflow | Trigger | Purpose |
|:---------|:--------|:--------|
| ci.yml | push, PR | Tests + coverage + benchmarks |
| security.yml | push, PR | Snyk + cargo-audit + cargo-deny |
| claude-analysis.yml | PR | AI-powered PR review |
| weekly-digest.yml | Monday 9am | Weekly Claude engineering digest |
| release.yml | tag | Build + publish + SBOM |
| load-test.yml | main only | k6 load tests |

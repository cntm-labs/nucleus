# CLAUDE.md — Nucleus Project Guide

## Overview
Nucleus is a production-grade authentication SaaS platform (like Clerk) built in Rust.

## Tech Stack
- **Language:** Rust (stable) with Axum web framework
- **Database:** PostgreSQL 16 (sqlx) + Redis 7 (sessions/cache)
- **Crypto:** argon2 (passwords), ring (CSPRNG/HMAC), jsonwebtoken (JWT), aes-gcm (encryption)
- **Frontend:** React 19 + Vite + TailwindCSS (admin dashboard)
- **SDKs:** 13 packages across Node.js, Next.js, Flutter, Swift, Android, Java, Python, Go, C#, Rust

## Build Commands
```sh
cargo check --workspace          # Type check
cargo test --workspace           # Run all tests (278)
cargo clippy --workspace -- -D warnings  # Lint
cargo fmt --all                  # Format
cargo deny check                 # License + advisory check
```

## Project Structure
```
server/                    # Single binary crate (modular monolith)
└── src/
    ├── main.rs            # Entry point
    ├── lib.rs             # Module declarations
    ├── core/              # Errors, types, crypto, validation
    ├── db/                # Repository traits + implementations
    ├── auth/              # Password, JWT, OAuth, MFA, passkeys
    ├── identity/          # User CRUD, ban/unban
    ├── org/               # Orgs, RBAC, invitations
    ├── session/           # Redis-backed sessions
    ├── webhook/           # HMAC signing, delivery
    ├── api/               # Dashboard API, project management
    ├── migrate/           # SQL migrations (15 files, 28+ tables)
    ├── config.rs          # Environment config
    ├── state.rs           # AppState composition
    ├── router.rs          # Axum route definitions
    ├── middleware/         # Auth, rate limiting, metrics
    ├── handlers/          # HTTP handler wrappers
    └── services/          # Email, SMS integrations
```

## Module Boundaries
- `core` → external deps only (leaf module)
- `db` → `core` only
- Feature modules (auth, identity, org, session, webhook, admin_api) → `core` + `db` only
- No cross-feature deps (auth ↛ org, identity ↛ auth)
- `main.rs` + `router` + `handlers` → all modules (composition root)

## Key Design Decisions
- **Hybrid sessions:** Short-lived JWT (5 min) + Redis sessions for revocation
- **Multi-tenancy:** Shared schema with project_id isolation
- **Data sovereignty:** Centralized (data in Nucleus) or Federated (data in project's DB)
- **Modular monolith:** Single binary, module boundaries = Rust modules within one crate

## Conventions
- All errors use AppError enum with code/status/message
- Repository pattern for DB access (traits for testability)
- Anti-enumeration: generic error messages for auth failures
- Constant-time comparison for all secrets
- No `#[allow(dead_code)]` — use it or remove it

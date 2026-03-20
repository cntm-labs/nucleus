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
cargo test --workspace           # Run all tests (215)
cargo clippy --workspace -- -D warnings  # Lint
cargo fmt --all                  # Format
cargo deny check                 # License + advisory check
```

## Project Structure
```
crates/
├── nucleus-server     # Binary: Axum router, middleware, config
├── nucleus-core       # Shared: errors, types, crypto, validation
├── nucleus-auth       # Auth: password, JWT, OAuth, MFA, passkeys
├── nucleus-identity   # Users: CRUD, ban/unban
├── nucleus-org        # Orgs: RBAC, invitations
├── nucleus-session    # Sessions: Redis-backed
├── nucleus-webhook    # Webhooks: HMAC signing, delivery
├── nucleus-admin-api  # Dashboard API: project management
├── nucleus-db         # DB: repository traits + implementations
└── nucleus-migrate    # Migrations: 13 SQL files, 28+ tables
```

## Dependency Rules (STRICT)
- nucleus-core → external deps only (leaf crate)
- nucleus-db → nucleus-core only
- Feature crates → nucleus-core + nucleus-db only
- No cross-feature deps (auth ↛ org, identity ↛ auth)
- nucleus-server → all crates (composition root)

## Key Design Decisions
- **Hybrid sessions:** Short-lived JWT (5 min) + Redis sessions for revocation
- **Multi-tenancy:** Shared schema with project_id isolation
- **Data sovereignty:** Centralized (data in Nucleus) or Federated (data in project's DB)
- **Modular monolith:** Single binary, module boundaries = crate boundaries

## Conventions
- All errors use AppError enum with code/status/message
- Repository pattern for DB access (traits for testability)
- Anti-enumeration: generic error messages for auth failures
- Constant-time comparison for all secrets
- No `#[allow(dead_code)]` — use it or remove it

<div align="center">

# nucleus

**High-performance, self-hosted authentication and user management platform built in Rust.**

> **Warning: DEV PREVIEW** — This project is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

[![CI](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml)
[![Security](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml)
[![Load Test](https://github.com/cntm-labs/nucleus/actions/workflows/load-test.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/load-test.yml)
[![Release Please](https://github.com/cntm-labs/nucleus/actions/workflows/release-please.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/release-please.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[![crates.io](https://img.shields.io/crates/v/cntm-nucleus?label=cntm-nucleus&color=fc8d62)](https://crates.io/crates/cntm-nucleus)
[![npm nucleus-node](https://img.shields.io/npm/v/@cntm-labs/nucleus-node?label=nucleus-node&color=cb3837)](https://www.npmjs.com/package/@cntm-labs/nucleus-node)
[![npm nucleus-js](https://img.shields.io/npm/v/@cntm-labs/nucleus-js?label=nucleus-js&color=cb3837)](https://www.npmjs.com/package/@cntm-labs/nucleus-js)
[![npm nucleus-react](https://img.shields.io/npm/v/@cntm-labs/nucleus-react?label=nucleus-react&color=cb3837)](https://www.npmjs.com/package/@cntm-labs/nucleus-react)
[![npm nucleus-nextjs](https://img.shields.io/npm/v/@cntm-labs/nucleus-nextjs?label=nucleus-nextjs&color=cb3837)](https://www.npmjs.com/package/@cntm-labs/nucleus-nextjs)
[![PyPI](https://img.shields.io/pypi/v/cntm-nucleus?label=cntm-nucleus&color=3775A9)](https://pypi.org/project/cntm-nucleus/)
[![pub.dev](https://img.shields.io/pub/v/cntm_nucleus?label=cntm_nucleus&color=02569B)](https://pub.dev/packages/cntm_nucleus)
[![NuGet](https://img.shields.io/nuget/v/Cntm.Nucleus?label=Cntm.Nucleus&color=004880)](https://www.nuget.org/packages/Cntm.Nucleus)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.cntm-labs/nucleus?label=nucleus&color=C71A36)](https://central.sonatype.com/artifact/io.github.cntm-labs/nucleus)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.cntm-labs/nucleus-android?label=nucleus-android&color=C71A36)](https://central.sonatype.com/artifact/io.github.cntm-labs/nucleus-android)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.cntm-labs/nucleus-java?label=nucleus-java&color=C71A36)](https://central.sonatype.com/artifact/io.github.cntm-labs/nucleus-java)
[![CocoaPods](https://img.shields.io/cocoapods/v/CntmNucleus?label=CntmNucleus&color=EE3322)](https://cocoapods.org/pods/CntmNucleus)
[![Go Reference](https://pkg.go.dev/badge/github.com/cntm-labs/nucleus/sdks/go.svg)](https://pkg.go.dev/github.com/cntm-labs/nucleus/sdks/go)

[![Rust](https://img.shields.io/badge/Rust-18k_LOC-dea584?logo=rust&logoColor=white)](crates/)
[![TypeScript](https://img.shields.io/badge/TypeScript-7k_LOC-3178C6?logo=typescript&logoColor=white)](sdks/)
[![Python](https://img.shields.io/badge/Python-0.4k_LOC-3775A9?logo=python&logoColor=white)](sdks/python/)
[![Dart](https://img.shields.io/badge/Dart-1.4k_LOC-02569B?logo=dart&logoColor=white)](sdks/flutter/)
[![Swift](https://img.shields.io/badge/Swift-1.2k_LOC-F05138?logo=swift&logoColor=white)](sdks/swift/)
[![Kotlin](https://img.shields.io/badge/Kotlin-1.4k_LOC-7F52FF?logo=kotlin&logoColor=white)](sdks/android/)
[![Java](https://img.shields.io/badge/Java-1.7k_LOC-ED8B00?logo=openjdk&logoColor=white)](sdks/java/)
[![Go](https://img.shields.io/badge/Go-0.9k_LOC-00ADD8?logo=go&logoColor=white)](sdks/go/)
[![C#](https://img.shields.io/badge/C%23-0.8k_LOC-512BD4?logo=dotnet&logoColor=white)](sdks/dotnet/)
[![SQL](https://img.shields.io/badge/SQL-0.4k_LOC-4479A1?logo=postgresql&logoColor=white)](crates/nucleus-migrate/)
[![Total Lines](https://img.shields.io/badge/Total-54k+_LOC-blue)](./)

[![Rust](https://img.shields.io/badge/Rust-dea584?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-dea584?logo=rust&logoColor=white)](https://github.com/tokio-rs/axum)
[![Tokio](https://img.shields.io/badge/Tokio-dea584?logo=rust&logoColor=white)](https://tokio.rs/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL_16-4169E1?logo=postgresql&logoColor=white)](https://www.postgresql.org/)
[![Redis](https://img.shields.io/badge/Redis_7-DC382D?logo=redis&logoColor=white)](https://redis.io/)
[![React](https://img.shields.io/badge/React_19-61DAFB?logo=react&logoColor=black)](https://react.dev/)
[![Vite](https://img.shields.io/badge/Vite-646CFF?logo=vite&logoColor=white)](https://vite.dev/)
[![TailwindCSS](https://img.shields.io/badge/Tailwind-06B6D4?logo=tailwindcss&logoColor=white)](https://tailwindcss.com/)

</div>

---

Full control over your auth infrastructure, your data, your rules.

## Why Nucleus?

- **Performance** — Built in Rust with Axum for minimal latency and maximum throughput
- **Security-first** — AES-GCM encryption at rest, constant-time secret comparison, anti-enumeration, PKCE for all OAuth flows
- **Self-hosted** — Deploy on your infrastructure, keep user data where you need it
- **Data sovereignty** — Centralized or Federated mode for full control over data location
- **Open source** — MIT licensed, no per-MAU pricing, no vendor lock-in

## Features

**Authentication** — Email/password, magic links, email OTP, OAuth (Google, GitHub, Microsoft, Apple, Discord, Facebook, LinkedIn, Slack, Twitter/X), passkeys/WebAuthn, SAML 2.0

**Multi-factor** — TOTP authenticator apps, SMS OTP (Twilio), email OTP (SendGrid), backup codes — all secrets encrypted at rest with AES-GCM

**Sessions** — Hybrid model: short-lived RS256 JWT (5 min) + Redis-backed sessions for instant revocation. Token hashing, constant-time comparison, JWT revocation list

**Organizations** — Multi-tenant RBAC with built-in roles (owner, admin, member), custom roles, 10 default permissions, invitations

**Webhooks** — 18 event types across user, session, org, MFA, and security categories. HMAC-SHA256 signing with replay protection. Exponential backoff retry

**Admin Dashboard** — Project management, OAuth provider config, API key management, signing key rotation, JWT templates, email templates, analytics (MAU, sign-ins, method breakdown), audit logs, billing/usage tracking

**Security** — Rate limiting (Redis sliding window), anti-enumeration, constant-time secret comparison, AES-GCM encryption at rest, OIDC discovery, PKCE for all OAuth flows

## SDKs

| SDK | Capability | Install | |
|-----|------------|---------|---|
| [![Crates.io](https://img.shields.io/crates/v/cntm-nucleus?label=Rust)](https://crates.io/crates/cntm-nucleus) | Server | `cargo add cntm-nucleus` | [Docs](sdks/rust/) |
| [![npm](https://img.shields.io/npm/v/@cntm-labs/nucleus-node?label=Node.js)](https://www.npmjs.com/package/@cntm-labs/nucleus-node) | Server | `npm i @cntm-labs/nucleus-node` | [Docs](sdks/node/) |
| [![PyPI](https://img.shields.io/pypi/v/cntm-nucleus?label=Python)](https://pypi.org/project/cntm-nucleus/) | Server | `pip install cntm-nucleus` | [Docs](sdks/python/) |
| [![Go Reference](https://pkg.go.dev/badge/github.com/cntm-labs/nucleus/sdks/go.svg)](https://pkg.go.dev/github.com/cntm-labs/nucleus/sdks/go) | Server | `go get github.com/cntm-labs/nucleus/sdks/go` | [Docs](sdks/go/) |
| [![NuGet](https://img.shields.io/nuget/v/Cntm.Nucleus?label=.NET)](https://www.nuget.org/packages/Cntm.Nucleus) | Server | `dotnet add package Cntm.Nucleus` | [Docs](sdks/dotnet/) |
| [![Maven Central](https://img.shields.io/maven-central/v/io.github.cntm-labs/nucleus?label=Java)](https://central.sonatype.com/artifact/io.github.cntm-labs/nucleus) | Server + Android | Maven: `io.github.cntm-labs:nucleus` | [Docs](sdks/java/) |
| [![npm](https://img.shields.io/npm/v/@cntm-labs/nucleus-js?label=JavaScript)](https://www.npmjs.com/package/@cntm-labs/nucleus-js) | Browser + Node | `npm i @cntm-labs/nucleus-js` | [Docs](sdks/js/) |
| [![npm](https://img.shields.io/npm/v/@cntm-labs/nucleus-nextjs?label=Next.js)](https://www.npmjs.com/package/@cntm-labs/nucleus-nextjs) | SSR + Client | `npm i @cntm-labs/nucleus-nextjs` | [Docs](sdks/nextjs/) |
| [![npm](https://img.shields.io/npm/v/@cntm-labs/nucleus-react?label=React)](https://www.npmjs.com/package/@cntm-labs/nucleus-react) | Client | `npm i @cntm-labs/nucleus-react` | [Docs](sdks/react/) |
| [![Pub](https://img.shields.io/pub/v/cntm_nucleus?label=Flutter)](https://pub.dev/packages/cntm_nucleus) | Client | `flutter pub add cntm_nucleus` | [Docs](sdks/flutter/) |
| [![CocoaPods](https://img.shields.io/cocoapods/v/CntmNucleus?label=Swift)](https://cocoapods.org/pods/CntmNucleus) | Client | `pod 'CntmNucleus'` | [Docs](sdks/swift/) |
| [![Maven Central](https://img.shields.io/maven-central/v/io.github.cntm-labs/nucleus-android?label=Android%20Kotlin)](https://central.sonatype.com/artifact/io.github.cntm-labs/nucleus-android) | Client | Gradle: `io.github.cntm-labs:nucleus-android` | [Docs](sdks/android/) |
| [![Maven Central](https://img.shields.io/maven-central/v/io.github.cntm-labs/nucleus-java?label=Android%20Java)](https://central.sonatype.com/artifact/io.github.cntm-labs/nucleus-java) | Client | Gradle: `io.github.cntm-labs:nucleus-java` | [Docs](sdks/android-java/) |

## Quick Start

### 1. Deploy Nucleus

```bash
# Requirements: PostgreSQL 16, Redis 7
git clone https://github.com/cntm-labs/nucleus.git
cd nucleus
cp .env.example .env  # Configure database, Redis, master key
cargo run --release
```

### 2. Create a Project

```bash
curl -X POST http://localhost:3000/api/v1/dashboard/projects \
  -H "Content-Type: application/json" \
  -d '{"name": "my-app", "data_mode": "centralized"}'
```

### 3. Add Auth to Your App

<details>
<summary><strong>React</strong></summary>

```tsx
import { NucleusProvider, useAuth } from '@cntm-labs/nucleus-react';

function App() {
  return (
    <NucleusProvider publishableKey="pk_...">
      <MyApp />
    </NucleusProvider>
  );
}

function MyApp() {
  const { isSignedIn, user } = useAuth();
  return <div>{isSignedIn ? `Hello ${user.email}` : 'Sign in'}</div>;
}
```
</details>

<details>
<summary><strong>Next.js</strong></summary>

```tsx
// app/layout.tsx
import { NucleusProvider } from '@cntm-labs/nucleus-nextjs';

export default function RootLayout({ children }) {
  return (
    <NucleusProvider publishableKey="pk_...">
      {children}
    </NucleusProvider>
  );
}
```
</details>

<details>
<summary><strong>Node.js</strong></summary>

```typescript
import { createNucleus } from '@cntm-labs/nucleus-node';

const nucleus = createNucleus({ secretKey: 'sk_...' });
const { userId } = await nucleus.verifySession(token);
```
</details>

<details>
<summary><strong>Python</strong></summary>

```python
from nucleus import NucleusClient

client = NucleusClient(secret_key="sk_...")
session = client.verify_session(token)
```
</details>

<details>
<summary><strong>Rust</strong></summary>

```rust
use cntm_nucleus::NucleusClient;

let client = NucleusClient::new("sk_...");
let claims = client.verify_session(&token).await?;
```
</details>

<details>
<summary><strong>Go</strong></summary>

```go
import nucleus "github.com/cntm-labs/nucleus/sdks/go"

client := nucleus.NewClient("sk_...")
claims, err := client.VerifySession(token)
```
</details>

<details>
<summary><strong>.NET</strong></summary>

```csharp
using Nucleus;

var client = new NucleusClient("sk_...");
var session = await client.VerifySessionAsync(token);
```
</details>

<details>
<summary><strong>Java</strong></summary>

```java
NucleusClient client = new NucleusClient("sk_...");
Session session = client.verifySession(token);
```
</details>

<details>
<summary><strong>Flutter</strong></summary>

```dart
import 'package:cntm_nucleus/cntm_nucleus.dart';

final nucleus = NucleusClient(publishableKey: 'pk_...');
final session = await nucleus.getSession();
```
</details>

<details>
<summary><strong>Swift</strong></summary>

```swift
import CntmNucleus

let nucleus = Nucleus(publishableKey: "pk_...")
let session = try await nucleus.getSession()
```
</details>

<details>
<summary><strong>Android (Kotlin)</strong></summary>

```kotlin
val nucleus = Nucleus.configure(context, publishableKey = "pk_...")
val session = nucleus.getSession()
```
</details>

<details>
<summary><strong>Android (Java)</strong></summary>

```java
Nucleus nucleus = Nucleus.configure(context, "pk_...");
Session session = nucleus.getSession();
```
</details>

## Architecture

Single Rust binary, module boundaries enforced at the crate level:

```
nucleus-server (Axum)
├── nucleus-core       errors, types, crypto, validation
├── nucleus-auth       password, JWT, OAuth, MFA, passkeys, SAML
├── nucleus-identity   user CRUD, ban/unban
├── nucleus-org        organizations, RBAC, invitations
├── nucleus-session    Redis-backed hybrid sessions
├── nucleus-webhook    HMAC signing, delivery, retry
├── nucleus-admin-api  dashboard API, analytics, billing
├── nucleus-db         repository traits + implementations
└── nucleus-migrate    SQL migrations (28+ tables)
```

## API Reference

Nucleus exposes a REST API under `/api/v1`:

| Area | Endpoints |
|------|-----------|
| Auth | Sign up, sign in, token refresh, sign out, sign out all |
| OAuth | Initiate OAuth flow, callback handler (9 providers) |
| Magic links | Send magic link, verify |
| OTP | Send email OTP, verify |
| MFA | Enroll TOTP/SMS, verify, manage backup codes |
| Passkeys | Registration + authentication ceremonies (WebAuthn) |
| Password | Reset request, confirm reset |
| Users | Profile CRUD, session management |
| Organizations | CRUD, members, roles, permissions, invitations |
| Admin | User management, ban/unban, webhooks, analytics |
| Dashboard | Projects, API keys, signing keys, OAuth config, templates |
| Discovery | `/.well-known/jwks.json`, `/.well-known/openid-configuration` |

## Contributing

```bash
cargo check --workspace          # Type check
cargo test --workspace           # Run all tests
cargo clippy --workspace -- -D warnings  # Lint
cargo fmt --all                  # Format
```

## License

[MIT](LICENSE)

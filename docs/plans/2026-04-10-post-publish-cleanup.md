# Post-Publish Cleanup — README, Restructure, Unified Releases

> **For Claude:** Execute these 3 tasks in order across separate sessions.

---

## Task 1: Update README.md (main + all SDK packages)

### Main README.md
Update the root README.md with:
- Installation instructions for all 13 published SDKs
- Links to each registry (use URLs from publish report)
- Quick start code examples per language
- Badge shields for each registry (crates.io, npm, PyPI, pub.dev, NuGet, Maven, CocoaPods, Go)

### SDK Package READMEs
Each SDK directory needs its own README.md with:
- Package-specific installation (`npm install`, `pip install`, `cargo add`, etc.)
- Quick start example in that language
- Link to main docs
- Registry badge

SDK directories to update:
```
sdks/rust/README.md
sdks/js/README.md
sdks/node/README.md
sdks/react/README.md
sdks/nextjs/README.md
sdks/python/README.md
sdks/flutter/README.md
sdks/dotnet/README.md
sdks/java/README.md
sdks/android/README.md
sdks/android-java/README.md
sdks/swift/README.md
sdks/go/README.md
```

### Registry URLs reference
| Package | Install Command | Registry URL |
|---------|----------------|--------------|
| `cntm-nucleus` (Rust) | `cargo add cntm-nucleus` | https://crates.io/crates/cntm-nucleus |
| `@cntm-labs/nucleus-js` | `npm i @cntm-labs/nucleus-js` | https://www.npmjs.com/package/@cntm-labs/nucleus-js |
| `@cntm-labs/nucleus-node` | `npm i @cntm-labs/nucleus-node` | https://www.npmjs.com/package/@cntm-labs/nucleus-node |
| `@cntm-labs/nucleus-react` | `npm i @cntm-labs/nucleus-react` | https://www.npmjs.com/package/@cntm-labs/nucleus-react |
| `@cntm-labs/nucleus-nextjs` | `npm i @cntm-labs/nucleus-nextjs` | https://www.npmjs.com/package/@cntm-labs/nucleus-nextjs |
| `cntm-nucleus` (Python) | `pip install cntm-nucleus` | https://pypi.org/project/cntm-nucleus/ |
| `cntm_nucleus` (Flutter) | `flutter pub add cntm_nucleus` | https://pub.dev/packages/cntm_nucleus |
| `Cntm.Nucleus` (.NET) | `dotnet add package Cntm.Nucleus` | https://www.nuget.org/packages/Cntm.Nucleus |
| `nucleus` (Java/Maven) | `<artifactId>nucleus</artifactId>` | https://central.sonatype.com/artifact/io.github.cntm-labs/nucleus |
| `nucleus-android` | Gradle: `io.github.cntm-labs:nucleus-android` | https://central.sonatype.com/artifact/io.github.cntm-labs/nucleus-android |
| `nucleus-java` | Gradle: `io.github.cntm-labs:nucleus-java` | https://central.sonatype.com/artifact/io.github.cntm-labs/nucleus-java |
| `CntmNucleus` (Swift) | `pod 'CntmNucleus'` | https://cocoapods.org/pods/CntmNucleus |
| Go module | `go get github.com/cntm-labs/nucleus/sdks/go` | https://pkg.go.dev/github.com/cntm-labs/nucleus/sdks/go |

---

## Task 2: Restructure `crates/*` Internal Modules

### Problem
9 internal crates under `crates/` are structured as independent packages but are NOT independent — they're tightly coupled internal modules of the server. They were accidentally published to crates.io (now yanked).

### Current structure
```
crates/
├── nucleus-core/        # errors, types, crypto
├── nucleus-db/          # repository traits + impls
├── nucleus-auth/        # password, JWT, OAuth, MFA
├── nucleus-identity/    # user CRUD
├── nucleus-org/         # orgs, RBAC
├── nucleus-session/     # Redis sessions
├── nucleus-webhook/     # webhook delivery
├── nucleus-admin-api/   # dashboard API
├── nucleus-migrate/     # SQL migrations
└── nucleus-server/      # Axum binary (depends on all above)
```

### Target structure
```
crates/
└── nucleus-server/
    └── src/
        ├── main.rs
        ├── core/        # was nucleus-core
        ├── db/          # was nucleus-db
        ├── auth/        # was nucleus-auth
        ├── identity/    # was nucleus-identity
        ├── org/         # was nucleus-org
        ├── session/     # was nucleus-session
        ├── webhook/     # was nucleus-webhook
        ├── admin_api/   # was nucleus-admin-api
        └── migrate/     # was nucleus-migrate
```

### Steps
1. Move each crate's `src/` contents into a module under `nucleus-server/src/`
2. Update all `use` imports (e.g. `nucleus_core::` → `crate::core::`)
3. Merge all dependencies into `nucleus-server/Cargo.toml`
4. Remove old crate directories and workspace members
5. Update CLAUDE.md project structure section
6. Run `cargo test --workspace` to verify nothing breaks

### Risks
- Large diff — do in a single PR with clear commit history
- Test imports may need updating
- The `sdks/rust` crate (`cntm-nucleus`) depends on `nucleus-core` types — check if it uses path dependency

---

## Task 3: Unified Version with release-please

### Problem
Current config creates 13 separate tags per release (one per SDK component). This clutters the repo and makes releases hard to manage.

### Current config
```json
"separate-pull-requests": true
// 13 packages × separate PRs × separate tags = chaos
```

### Target config
```json
"separate-pull-requests": false,
"group-pull-request-title-pattern": "chore: release v${version}"
```

All SDKs share one version, one PR, one tag (`v0.3.0`).

### Steps
1. Update `release-please-config.json`:
   - Set `"separate-pull-requests": false`
   - Add `"group-pull-request-title-pattern": "chore: release v${version}"`
2. Align all versions in `.release-please-manifest.json` to the same value
3. Update `release-please.yml` workflow:
   - Simplify version extraction (single version instead of per-component)
   - Remove per-component output mapping
4. Update `sdk-publish.yml`:
   - Single version input from the unified release
5. Clean up old tags/releases that are per-component format
6. Test by pushing a conventional commit and verifying single PR + single tag

### Tag cleanup
Delete all per-component tags and keep only unified ones:
```
# Delete these patterns:
@cntm-labs/nucleus-*-v*
cntm-nucleus-*-v*
Cntm.Nucleus-v*
CntmNucleus-v*

# Keep/create:
v0.2.0
```

---

## Execution Order
1. **Task 1 (README)** — safe, no breaking changes
2. **Task 3 (unified version)** — do before Task 2 so the restructure gets released cleanly
3. **Task 2 (restructure)** — biggest change, do last with clean release pipeline

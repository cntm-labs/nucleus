# Package Registry Publishing Design

## Goal
Publish all 13 Nucleus SDKs to their respective package registries as **dev preview** packages (`0.1.0-dev.1`), with clear warnings that they are not production-ready.

## Naming Convention

| Registry | Package Name | Version |
|----------|-------------|---------|
| npm | `@cntm-labs/node` | `0.1.0-dev.1` |
| npm | `@cntm-labs/nextjs` | `0.1.0-dev.1` |
| npm | `@cntm-labs/react` | `0.1.0-dev.1` |
| npm | `@cntm-labs/js` | `0.1.0-dev.1` |
| PyPI | `cntm-labs-nucleus` | `0.1.0.dev1` |
| crates.io | `nucleus-rs` | `0.1.0-dev.1` |
| pub.dev | `nucleus_flutter` | `0.1.0-dev.1` |
| NuGet | `CntmLabs.Nucleus` / `CntmLabs.Nucleus.AspNetCore` | `0.1.0-dev.1` |
| Maven Central | `io.github.cntm-labs:nucleus-java` | `0.1.0-dev.1` |
| Maven Central | `io.github.cntm-labs:nucleus-android` | `0.1.0-dev.1` |
| Maven Central | `io.github.cntm-labs:nucleus-android-java` | `0.1.0-dev.1` |
| Swift PM | `https://github.com/cntm-labs/nucleus.git` | tag `0.1.0-dev.1` |
| CocoaPods | `NucleusSwift` | `0.1.0-dev.1` |
| Go modules | `github.com/cntm-labs/nucleus/sdks/go` | tag `sdks/go/v0.1.0-dev.1` |

**PyPI note:** Uses `.dev1` not `-dev.1` per PEP 440.

## Dev Warning Strategy (3 layers)

### Layer 1: Version Number
`0.1.0-dev.1` — semver pre-release tag. Package managers will NOT install by default; users must specify the exact version.

### Layer 2: README Banner
Every SDK README starts with:
```markdown
> **WARNING: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).
```

### Layer 3: Runtime Warning
SDKs with runtime initialization print on first use:
```
[Nucleus] WARNING: You are using a dev preview (0.1.0-dev.1).
Do not use in production. APIs may change without notice.
```
Removed when version no longer contains `-dev`.

## CI/CD: publish-sdks.yml

Single workflow with manual dispatch:
```yaml
on:
  workflow_dispatch:
    inputs:
      sdk: [all, npm, python, rust, flutter, nuget, maven, swift, go]
      version: '0.1.0-dev.1'
```

### Publish order
1. **Go** — git tag only (no registry)
2. **npm** — js → node → react → nextjs (dependency order)
3. **Rust** — `cargo publish`
4. **Python** — build wheel + `twine upload`
5. **Flutter** — `dart pub publish --force`
6. **NuGet** — `dotnet pack` + `dotnet nuget push`
7. **Maven** — `mvn deploy` with GPG signing (Java + Android x2)
8. **Swift** — `pod trunk push` (CocoaPods) + git tag (SPM)

## Required Secrets (all configured)

| Secret | Registry |
|--------|----------|
| `NPM_TOKEN` | npm |
| `PYPI_TOKEN` | PyPI |
| `CRATES_IO_TOKEN` | crates.io |
| `PUB_DEV_CREDENTIALS` | pub.dev |
| `NUGET_API_KEY` | NuGet |
| `MAVEN_GPG_KEY` | Maven Central |
| `MAVEN_GPG_PASSPHRASE` | Maven Central |
| `MAVEN_USERNAME` | Maven Central |
| `MAVEN_PASSWORD` | Maven Central |
| `COCOAPODS_TRUNK_TOKEN` | CocoaPods |

## Package Metadata Requirements

Every SDK package config must include:
- **Name** — as listed in naming convention
- **Version** — `0.1.0-dev.1` (or `.dev1` for PyPI)
- **Description** — starts with "DEV PREVIEW — Not ready for production."
- **License** — MIT
- **Repository** — `https://github.com/cntm-labs/nucleus`
- **Homepage** — `https://github.com/cntm-labs/nucleus`
- **Author** — `cntm-labs`

## Package Manager Compatibility

npm packages work across all JS package managers:
- `npm i @cntm-labs/node@0.1.0-dev.1`
- `yarn add @cntm-labs/node@0.1.0-dev.1`
- `pnpm add @cntm-labs/node@0.1.0-dev.1`
- `bun add @cntm-labs/node@0.1.0-dev.1`

## Version Management — Release Please

### Strategy: Independent Versions
Each package has its own version and CHANGELOG. Updating Flutter SDK does not bump Node.js SDK.

### Tool: Release Please (Google)
GitHub Action that reads conventional commits and creates Release PRs automatically.

**How it works:**
1. Developer commits with conventional format: `feat(node): add session middleware`
2. Release Please detects the change belongs to `@cntm-labs/node`
3. Creates/updates a Release PR with version bump + CHANGELOG
4. Maintainer reviews and merges the Release PR
5. Release Please creates a GitHub Release + git tag
6. `publish-sdks.yml` triggers on the new tag → publishes to registry

### Conventional Commit Scopes
Each component has a scope that maps to a package:

| Scope | Package | CHANGELOG Location |
|-------|---------|-------------------|
| `server` | nucleus-server | `crates/nucleus-server/CHANGELOG.md` |
| `node` | @cntm-labs/node | `sdks/node/CHANGELOG.md` |
| `nextjs` | @cntm-labs/nextjs | `sdks/nextjs/CHANGELOG.md` |
| `react` | @cntm-labs/react | `sdks/react/CHANGELOG.md` |
| `js` | @cntm-labs/js | `sdks/js/CHANGELOG.md` |
| `python` | cntm-labs-nucleus | `sdks/python/CHANGELOG.md` |
| `rust-sdk` | nucleus-rs | `sdks/rust/CHANGELOG.md` |
| `flutter` | nucleus_flutter | `sdks/flutter/CHANGELOG.md` |
| `dotnet` | CntmLabs.Nucleus | `sdks/dotnet/CHANGELOG.md` |
| `java` | nucleus-java | `sdks/java/CHANGELOG.md` |
| `android` | nucleus-android | `sdks/android/CHANGELOG.md` |
| `android-java` | nucleus-android-java | `sdks/android-java/CHANGELOG.md` |
| `swift` | NucleusSwift | `sdks/swift/CHANGELOG.md` |
| `go` | nucleus/sdks/go | `sdks/go/CHANGELOG.md` |
| `dashboard` | nucleus-dashboard | `dashboard/CHANGELOG.md` |

### Version Bump Rules (semver)
- `fix(node): ...` → patch (0.1.0 → 0.1.1)
- `feat(node): ...` → minor (0.1.0 → 0.2.0)
- `feat(node)!: ...` or `BREAKING CHANGE:` → major (0.1.0 → 1.0.0)
- `chore(node): ...` → no bump (internal changes)

### Release Please Config Files
- `release-please-config.json` — defines all packages, their paths, and release types
- `.release-please-manifest.json` — tracks current version of each package

### Workflow: release-please.yml
```yaml
on:
  push:
    branches: [main]
```
Runs on every push to main. Creates/updates Release PRs. When merged, creates tags and triggers publish.

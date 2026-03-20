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

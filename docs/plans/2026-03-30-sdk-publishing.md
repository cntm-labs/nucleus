# Phase 3: SDK Publishing — Registry Readiness

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make all 13 SDKs ready for publishing to their respective package registries (npm, pub.dev, PyPI, Maven, NuGet, crates.io, pkg.go.dev, SPM/CocoaPods).

**Architecture:** Fixes are grouped by registry type to minimize context switching. Cross-cutting issues (LICENSE files, description cleanup) are done first in a single pass. Each registry group is then fixed independently. No source code changes — only package metadata, build config, and publishing setup.

**Tech Stack:** npm (tsup), pub.dev (dart), PyPI (hatchling), Maven (gradle/pom), NuGet (.csproj), crates.io (cargo), Go modules, SPM/CocoaPods

**Execution order:** Cross-cutting → npm (4 SDKs) → Python → Flutter → Go → Android (2 SDKs) → Java → .NET → Swift → Rust

---

## Task 1: Add LICENSE Files to All SDKs

### Problem
Only Flutter has its own LICENSE file. Every other SDK will publish without a license file, which violates registry best practices and may block publishing on some registries.

### Files
- Create: `sdks/js/LICENSE`
- Create: `sdks/react/LICENSE`
- Create: `sdks/nextjs/LICENSE`
- Create: `sdks/node/LICENSE`
- Create: `sdks/python/LICENSE`
- Create: `sdks/go/LICENSE`
- Create: `sdks/android/LICENSE`
- Create: `sdks/android-java/LICENSE`
- Create: `sdks/java/LICENSE`
- Create: `sdks/dotnet/LICENSE`
- Create: `sdks/swift/LICENSE`
- Create: `sdks/rust/LICENSE`

### Step 1: Copy root LICENSE to all SDK directories

```bash
for sdk in js react nextjs node python go android android-java java dotnet swift rust; do
  cp LICENSE sdks/$sdk/LICENSE
done
```

### Step 2: Verify all exist

```bash
ls sdks/*/LICENSE | wc -l
# Expected: 13 (including flutter)
```

### Step 3: Commit

```bash
git add sdks/*/LICENSE
git commit -m "chore: add LICENSE file to all SDK directories"
```

---

## Task 2: Fix npm SDKs (JS, React, Next.js, Node)

### Problem
All 4 npm SDKs are missing: `prepublishOnly` script, `keywords`, `sideEffects: false`, LICENSE in `files` array. React SDK is missing `react-dom` in peerDependencies.

### Files
- Modify: `sdks/js/package.json`
- Modify: `sdks/react/package.json`
- Modify: `sdks/nextjs/package.json`
- Modify: `sdks/node/package.json`

### Step 1: Update sdks/js/package.json

Add/modify these fields:

```json
{
  "description": "Nucleus authentication SDK for JavaScript.",
  "keywords": ["authentication", "auth", "nucleus", "sdk", "jwt", "oauth"],
  "sideEffects": false,
  "files": ["dist", "LICENSE", "README.md"],
  "scripts": {
    "build": "tsup",
    "test": "vitest run",
    "prepublishOnly": "npm run build"
  }
}
```

Changes:
- Remove "DEV PREVIEW — Not ready for production." prefix from `description`
- Add `keywords` array
- Add `sideEffects: false` for tree-shaking
- Add `LICENSE` and `README.md` to `files` array
- Add `prepublishOnly` script that runs build before publish

### Step 2: Update sdks/react/package.json

Same changes as JS, plus:

```json
{
  "description": "Nucleus authentication SDK for React.",
  "keywords": ["authentication", "auth", "nucleus", "sdk", "react", "hooks"],
  "peerDependencies": {
    "react": ">=18",
    "react-dom": ">=18"
  }
}
```

Add `react-dom` to peerDependencies.

### Step 3: Update sdks/nextjs/package.json

```json
{
  "description": "Nucleus authentication SDK for Next.js.",
  "keywords": ["authentication", "auth", "nucleus", "sdk", "nextjs", "server-components"]
}
```

Plus same `sideEffects`, `files`, `prepublishOnly` changes.

### Step 4: Update sdks/node/package.json

```json
{
  "description": "Nucleus authentication SDK for Node.js.",
  "keywords": ["authentication", "auth", "nucleus", "sdk", "node", "express", "fastify"]
}
```

Plus same changes. Node already has `engines` — keep it.

### Step 5: Verify builds work

```bash
cd sdks/js && npm run build && ls dist/
cd ../react && npm run build && ls dist/
cd ../nextjs && npm run build && ls dist/
cd ../node && npm run build && ls dist/
```

All should produce `dist/esm/` and `dist/cjs/` directories.

### Step 6: Verify pack output includes LICENSE

```bash
cd sdks/js && npm pack --dry-run 2>&1 | grep LICENSE
# Expected: LICENSE is listed in the tarball contents
```

### Step 7: Add dist/ to .gitignore if not already

Ensure `sdks/*/dist/` is in `.gitignore` so built artifacts aren't committed.

### Step 8: Commit

```bash
git add sdks/js/package.json sdks/react/package.json sdks/nextjs/package.json sdks/node/package.json
git commit -m "chore(npm): add prepublishOnly, keywords, sideEffects, LICENSE to all npm SDKs"
```

---

## Task 3: Fix Python SDK

### Problem
`pyproject.toml` has `build-backend = "hatchling.backends"` — **this is wrong**. The correct value is `hatchling.build`. `python -m build` will fail with `ModuleNotFoundError`. Also missing keywords and classifiers.

### Files
- Modify: `sdks/python/pyproject.toml`

### Step 1: Fix build-backend

```toml
# OLD (line 3):
build-backend = "hatchling.backends"

# NEW:
build-backend = "hatchling.build"
```

### Step 2: Update description and add keywords

```toml
description = "Nucleus authentication SDK for Python."
keywords = ["authentication", "auth", "nucleus", "sdk", "jwt", "oauth"]
classifiers = [
    "Development Status :: 3 - Alpha",
    "License :: OSI Approved :: MIT License",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.10",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
    "Framework :: FastAPI",
    "Framework :: Django",
    "Framework :: Flask",
    "Topic :: Security",
]
```

### Step 3: Verify build works

```bash
cd sdks/python
pip install hatchling build
python -m build --sdist --wheel
ls dist/
# Expected: .tar.gz and .whl files
```

### Step 4: Commit

```bash
git commit -m "fix(python): fix build-backend to hatchling.build and add metadata"
```

---

## Task 4: Fix Flutter SDK

### Problem
Missing `topics` field (pub.dev discoverability). Description has "DEV PREVIEW" prefix.

### Files
- Modify: `sdks/flutter/pubspec.yaml`

### Step 1: Update pubspec.yaml

```yaml
description: "Nucleus authentication SDK for Flutter."
topics:
  - authentication
  - auth
  - sdk
```

Remove the "DEV PREVIEW" prefix from description.

### Step 2: Verify

```bash
cd sdks/flutter
dart pub publish --dry-run
```

### Step 3: Commit

```bash
git commit -m "chore(flutter): add topics and clean description for pub.dev"
```

---

## Task 5: Fix Go SDK

### Problem
Missing `go.sum` file — consumers cannot verify dependency checksums. Missing `go.sum` also causes issues with `go get`.

### Files
- Generate: `sdks/go/go.sum`

### Step 1: Generate go.sum

```bash
cd sdks/go
go mod tidy
```

This will fetch dependencies and generate `go.sum`.

### Step 2: Verify module works

```bash
go build ./...
```

### Step 3: Commit

```bash
git add go.mod go.sum
git commit -m "chore(go): generate go.sum for dependency verification"
```

---

## Task 6: Fix Android SDKs (Kotlin + Java)

### Problem
Both Android SDKs are missing `settings.gradle.kts`/`settings.gradle` (build won't work standalone), missing `publishing {}` block (can't publish to Maven), and missing POM metadata.

### Files
- Create: `sdks/android/settings.gradle.kts`
- Modify: `sdks/android/build.gradle.kts` — add publishing block
- Create: `sdks/android-java/settings.gradle`
- Modify: `sdks/android-java/build.gradle` — add publishing block

### Step 1: Create settings.gradle.kts for Android Kotlin SDK

```kotlin
// sdks/android/settings.gradle.kts
rootProject.name = "nucleus-android"
```

### Step 2: Add publishing block to Android Kotlin SDK

Add at the end of `sdks/android/build.gradle.kts`:

```kotlin
afterEvaluate {
    publishing {
        publications {
            create<MavenPublication>("release") {
                from(components["release"])
                groupId = "io.github.cntm-labs"
                artifactId = "nucleus-android"
                version = project.version.toString()

                pom {
                    name.set("Nucleus Android SDK")
                    description.set("Nucleus authentication SDK for Android (Kotlin).")
                    url.set("https://github.com/cntm-labs/nucleus")
                    licenses {
                        license {
                            name.set("MIT")
                            url.set("https://opensource.org/licenses/MIT")
                        }
                    }
                    developers {
                        developer {
                            id.set("cntm-labs")
                            name.set("cntm-labs")
                            url.set("https://github.com/cntm-labs")
                        }
                    }
                    scm {
                        connection.set("scm:git:git://github.com/cntm-labs/nucleus.git")
                        url.set("https://github.com/cntm-labs/nucleus")
                    }
                }
            }
        }
    }
}
```

### Step 3: Create settings.gradle for Android Java SDK

```groovy
// sdks/android-java/settings.gradle
rootProject.name = 'nucleus-android-java'
```

### Step 4: Add publishing block to Android Java SDK

Add at the end of `sdks/android-java/build.gradle`:

```groovy
afterEvaluate {
    publishing {
        publications {
            release(MavenPublication) {
                from components.release
                groupId = 'io.github.cntm-labs'
                artifactId = 'nucleus-android-java'
                version = project.version

                pom {
                    name = 'Nucleus Android Java SDK'
                    description = 'Nucleus authentication SDK for Android (Java).'
                    url = 'https://github.com/cntm-labs/nucleus'
                    licenses {
                        license {
                            name = 'MIT'
                            url = 'https://opensource.org/licenses/MIT'
                        }
                    }
                    developers {
                        developer {
                            id = 'cntm-labs'
                            name = 'cntm-labs'
                            url = 'https://github.com/cntm-labs'
                        }
                    }
                    scm {
                        connection = 'scm:git:git://github.com/cntm-labs/nucleus.git'
                        url = 'https://github.com/cntm-labs/nucleus'
                    }
                }
            }
        }
    }
}
```

### Step 5: Update descriptions

Remove "DEV PREVIEW" prefix from any description strings in both build files.

### Step 6: Commit

```bash
git commit -m "chore(android): add settings.gradle and Maven publishing blocks for both Android SDKs"
```

---

## Task 7: Fix Java (Maven) SDK

### Problem
Missing `maven-source-plugin`, `maven-javadoc-plugin`, `maven-gpg-plugin`, and `distributionManagement`. Maven Central requires all of these.

### Files
- Modify: `sdks/java/pom.xml`

### Step 1: Add build plugins and distributionManagement

Add inside `<project>` after `</dependencies>`:

```xml
  <build>
    <plugins>
      <plugin>
        <groupId>org.apache.maven.plugins</groupId>
        <artifactId>maven-source-plugin</artifactId>
        <version>3.3.1</version>
        <executions>
          <execution>
            <id>attach-sources</id>
            <goals><goal>jar-no-fork</goal></goals>
          </execution>
        </executions>
      </plugin>
      <plugin>
        <groupId>org.apache.maven.plugins</groupId>
        <artifactId>maven-javadoc-plugin</artifactId>
        <version>3.11.2</version>
        <executions>
          <execution>
            <id>attach-javadocs</id>
            <goals><goal>jar</goal></goals>
          </execution>
        </executions>
      </plugin>
      <plugin>
        <groupId>org.apache.maven.plugins</groupId>
        <artifactId>maven-gpg-plugin</artifactId>
        <version>3.2.7</version>
        <executions>
          <execution>
            <id>sign-artifacts</id>
            <phase>verify</phase>
            <goals><goal>sign</goal></goals>
          </execution>
        </executions>
      </plugin>
      <plugin>
        <groupId>org.sonatype.central</groupId>
        <artifactId>central-publishing-maven-plugin</artifactId>
        <version>0.7.0</version>
        <extensions>true</extensions>
        <configuration>
          <publishingServerId>central</publishingServerId>
        </configuration>
      </plugin>
    </plugins>
  </build>
```

### Step 2: Update description

```xml
<description>Nucleus authentication SDK for Java/Spring Boot.</description>
```

### Step 3: Commit

```bash
git commit -m "chore(java): add Maven Central publishing plugins (source, javadoc, gpg, central)"
```

---

## Task 8: Fix .NET SDKs

### Problem
Targeting only `net8.0` — excludes LTS users on net6.0. Missing SourceLink for debugging. Description has "DEV PREVIEW" prefix.

### Files
- Modify: `sdks/dotnet/src/Nucleus/Nucleus.csproj`
- Modify: `sdks/dotnet/src/Nucleus.AspNetCore/Nucleus.AspNetCore.csproj`

### Step 1: Add multi-targeting and SourceLink to Nucleus.csproj

```xml
<PropertyGroup>
    <TargetFrameworks>net6.0;net8.0</TargetFrameworks>
    <Description>Nucleus authentication SDK for .NET.</Description>
    <!-- SourceLink -->
    <PublishRepositoryUrl>true</PublishRepositoryUrl>
    <EmbedUntrackedSources>true</EmbedUntrackedSources>
    <IncludeSymbols>true</IncludeSymbols>
    <SymbolPackageFormat>snupkg</SymbolPackageFormat>
</PropertyGroup>

<ItemGroup>
    <PackageReference Include="Microsoft.SourceLink.GitHub" Version="8.0.0" PrivateAssets="All"/>
</ItemGroup>
```

Change `<TargetFramework>net8.0</TargetFramework>` to `<TargetFrameworks>net6.0;net8.0</TargetFrameworks>` (note the plural).

### Step 2: Same for Nucleus.AspNetCore.csproj

Multi-target and add SourceLink. Note: `<FrameworkReference Include="Microsoft.AspNetCore.App" />` works for both net6.0 and net8.0.

### Step 3: Verify build

```bash
cd sdks/dotnet
dotnet build
dotnet pack
```

### Step 4: Commit

```bash
git commit -m "chore(dotnet): add multi-targeting (net6.0+net8.0) and SourceLink"
```

---

## Task 9: Fix Swift SDK

### Problem
CocoaPods podspec uses version `0.1.0-dev.1` which may fail `pod spec lint`. Package.swift only targets iOS — missing macOS. No LICENSE in SDK dir (fixed in Task 1).

### Files
- Modify: `sdks/swift/NucleusSwift.podspec`
- Modify: `sdks/swift/Package.swift`

### Step 1: Fix podspec version and license path

```ruby
Pod::Spec.new do |s|
  s.name         = 'NucleusSwift'
  s.version      = '0.1.0.dev1'
  s.summary      = 'Nucleus authentication SDK for iOS/macOS.'
  s.homepage     = 'https://github.com/cntm-labs/nucleus'
  s.license      = { :type => 'MIT', :file => 'LICENSE' }
  s.author       = { 'cntm-labs' => 'dev@cntm-labs.dev' }
  s.source       = { :git => 'https://github.com/cntm-labs/nucleus.git', :tag => s.version.to_s }
  s.ios.deployment_target = '16.0'
  s.osx.deployment_target = '13.0'
  s.swift_version = '5.9'
  s.source_files = 'Sources/NucleusSwift/**/*.swift'
end
```

Changes:
- Version: `0.1.0-dev.1` → `0.1.0.dev1` (CocoaPods compatible)
- License file: `../../LICENSE` → `LICENSE` (now has local copy from Task 1)
- Description: Remove "DEV PREVIEW" prefix
- Add macOS deployment target

### Step 2: Add macOS platform to Package.swift

```swift
platforms: [
    .iOS(.v16),
    .macOS(.v13),
],
```

### Step 3: Commit

```bash
git commit -m "chore(swift): fix podspec version format and add macOS platform support"
```

---

## Task 10: Fix Rust SDK

### Problem
Missing LICENSE file in SDK dir (fixed in Task 1). Missing `documentation` field. Description has "DEV PREVIEW" prefix.

### Files
- Modify: `sdks/rust/Cargo.toml`

### Step 1: Update Cargo.toml

```toml
description = "Nucleus authentication SDK for Rust."
documentation = "https://docs.rs/nucleus-rs"
```

Remove "DEV PREVIEW" prefix from description. Add `documentation` field.

### Step 2: Verify

```bash
cd sdks/rust
cargo package --list
# Verify LICENSE is included
```

### Step 3: Commit

```bash
git commit -m "chore(rust): add documentation URL and clean description for crates.io"
```

---

## Task 11: Update .env.example

### Problem
`MASTER_ENCRYPTION_KEY` value `change-me-in-production-64-hex-chars` is not valid hex and will crash the server on first run.

### Files
- Modify: `.env.example`

### Step 1: Replace with valid hex placeholder

```bash
# IMPORTANT: Replace this with a real 64-character hex key in production!
# Generate with: openssl rand -hex 32
MASTER_ENCRYPTION_KEY=0000000000000000000000000000000000000000000000000000000000000000
```

### Step 2: Commit

```bash
git commit -m "fix: use valid hex placeholder in .env.example for MASTER_ENCRYPTION_KEY"
```

---

## Verification Checklist

After all tasks complete:

```bash
# Verify all SDKs have LICENSE
ls sdks/*/LICENSE | wc -l  # Expected: 13

# Verify npm builds work
for sdk in js react nextjs node; do
  echo "=== $sdk ===" && cd sdks/$sdk && npm run build && cd ../..
done

# Verify no "DEV PREVIEW" in package descriptions
grep -r "DEV PREVIEW" sdks/*/package.json sdks/*/pubspec.yaml sdks/*/pyproject.toml sdks/*/Cargo.toml sdks/*/*.podspec sdks/*/pom.xml sdks/*/src/*/*.csproj 2>/dev/null

# Verify Python build works
cd sdks/python && python -m build --sdist 2>&1 | tail -3

# Verify Go module
cd sdks/go && go mod tidy && go build ./...

# Verify .NET builds
cd sdks/dotnet && dotnet build

# Verify Rust packages
cd sdks/rust && cargo package --list | grep LICENSE
```

All must pass before PR.

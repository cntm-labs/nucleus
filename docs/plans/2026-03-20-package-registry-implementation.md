# Package Registry Publishing — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Publish all 13 Nucleus SDKs to their respective package registries as `0.1.0-dev.1` dev previews with clear production warnings.

**Architecture:** Update each SDK's package config with correct naming (`@cntm-labs/*`), metadata (license, repo URL, description), and version (`0.1.0-dev.1`). Add dev warning banners to READMEs and runtime warnings. Create a single `publish-sdks.yml` GitHub Actions workflow with per-registry jobs triggered by manual dispatch.

**Tech Stack:** npm, PyPI (twine), crates.io (cargo), pub.dev (dart), NuGet (dotnet), Maven Central (mvn), CocoaPods (pod), Go modules (git tag), GitHub Actions

---

### Task 1: Update npm package configs (4 packages)

**Files:**
- Modify: `sdks/js/package.json`
- Modify: `sdks/node/package.json`
- Modify: `sdks/react/package.json`
- Modify: `sdks/nextjs/package.json`

**Step 1: Update all 4 package.json files**

Each package.json needs these fields added/changed:
- `"name"`: change from `@nucleus/*` to `@cntm-labs/*`
- `"version"`: change from `"0.1.0"` to `"0.1.0-dev.1"`
- `"description"`: prepend `"DEV PREVIEW — Not ready for production."`
- Add `"license": "MIT"`
- Add `"author": "cntm-labs"`
- Add `"repository": { "type": "git", "url": "https://github.com/cntm-labs/nucleus.git", "directory": "sdks/<name>" }`
- Add `"homepage": "https://github.com/cntm-labs/nucleus"`
- Add `"publishConfig": { "access": "public" }`

Example for `sdks/js/package.json`:
```json
{
  "name": "@cntm-labs/js",
  "version": "0.1.0-dev.1",
  "description": "DEV PREVIEW — Not ready for production. Nucleus vanilla JS authentication SDK.",
  "license": "MIT",
  "author": "cntm-labs",
  "repository": {
    "type": "git",
    "url": "https://github.com/cntm-labs/nucleus.git",
    "directory": "sdks/js"
  },
  "homepage": "https://github.com/cntm-labs/nucleus",
  "publishConfig": { "access": "public" }
}
```

Apply same pattern to `node`, `react`, `nextjs` with appropriate directory and description.

**Step 2: Verify JSON is valid**

Run: `cd sdks/js && node -e "require('./package.json')" && cd ../node && node -e "require('./package.json')" && cd ../react && node -e "require('./package.json')" && cd ../nextjs && node -e "require('./package.json')"`
Expected: no output (no errors)

**Step 3: Commit**

```bash
git add sdks/js/package.json sdks/node/package.json sdks/react/package.json sdks/nextjs/package.json
git commit -m "chore(sdks): update npm package configs for @cntm-labs scope"
```

---

### Task 2: Update Python package config

**Files:**
- Modify: `sdks/python/pyproject.toml`

**Step 1: Update pyproject.toml**

Change name, version, and add metadata:
```toml
[project]
name = "cntm-labs-nucleus"
version = "0.1.0.dev1"
description = "DEV PREVIEW — Not ready for production. Nucleus authentication SDK for Python."
license = {text = "MIT"}
authors = [{name = "cntm-labs", email = "dev@cntm-labs.dev"}]
readme = "README.md"
requires-python = ">=3.10"
classifiers = [
    "Development Status :: 2 - Pre-Alpha",
    "License :: OSI Approved :: MIT License",
    "Programming Language :: Python :: 3",
]

[project.urls]
Homepage = "https://github.com/cntm-labs/nucleus"
Repository = "https://github.com/cntm-labs/nucleus"
Issues = "https://github.com/cntm-labs/nucleus/issues"
```

Note: PyPI uses `.dev1` not `-dev.1` per PEP 440.

**Step 2: Verify toml is valid**

Run: `cd sdks/python && python3 -c "import tomllib; tomllib.load(open('pyproject.toml','rb')); print('OK')"`
Expected: `OK`

**Step 3: Commit**

```bash
git add sdks/python/pyproject.toml
git commit -m "chore(sdks): update Python package config for cntm-labs-nucleus"
```

---

### Task 3: Update Rust SDK config

**Files:**
- Modify: `sdks/rust/Cargo.toml`

**Step 1: Update Cargo.toml**

Change version and ensure metadata is complete:
```toml
[package]
name = "nucleus-rs"
version = "0.1.0-dev.1"
edition = "2021"
license = "MIT"
description = "DEV PREVIEW — Not ready for production. Nucleus authentication SDK for Rust."
repository = "https://github.com/cntm-labs/nucleus"
homepage = "https://github.com/cntm-labs/nucleus"
readme = "README.md"
keywords = ["authentication", "auth", "nucleus", "sdk"]
categories = ["authentication", "web-programming"]
```

**Step 2: Verify**

Run: `cd sdks/rust && cargo check 2>&1 | tail -1`
Expected: `Finished`

**Step 3: Commit**

```bash
git add sdks/rust/Cargo.toml
git commit -m "chore(sdks): update Rust SDK config for 0.1.0-dev.1"
```

---

### Task 4: Update Flutter SDK config

**Files:**
- Modify: `sdks/flutter/pubspec.yaml`

**Step 1: Update pubspec.yaml**

```yaml
name: nucleus_flutter
description: "DEV PREVIEW — Not ready for production. Nucleus authentication SDK for Flutter."
version: 0.1.0-dev.1
homepage: https://github.com/cntm-labs/nucleus
repository: https://github.com/cntm-labs/nucleus
issue_tracker: https://github.com/cntm-labs/nucleus/issues

environment:
  sdk: '>=3.0.0 <4.0.0'
  flutter: '>=3.16.0'
```

Keep existing dependencies unchanged.

**Step 2: Verify**

Run: `cd sdks/flutter && flutter pub get 2>&1 | tail -1`
Expected: no errors

**Step 3: Commit**

```bash
git add sdks/flutter/pubspec.yaml
git commit -m "chore(sdks): update Flutter SDK config for 0.1.0-dev.1"
```

---

### Task 5: Update .NET SDK configs (2 packages)

**Files:**
- Modify: `sdks/dotnet/src/Nucleus/Nucleus.csproj`
- Modify: `sdks/dotnet/src/Nucleus.AspNetCore/Nucleus.AspNetCore.csproj`

**Step 1: Update both .csproj files**

Add PropertyGroup metadata to `Nucleus.csproj`:
```xml
<PropertyGroup>
  <TargetFramework>net8.0</TargetFramework>
  <PackageId>CntmLabs.Nucleus</PackageId>
  <Version>0.1.0-dev.1</Version>
  <Authors>cntm-labs</Authors>
  <Description>DEV PREVIEW — Not ready for production. Nucleus authentication SDK for .NET.</Description>
  <PackageLicenseExpression>MIT</PackageLicenseExpression>
  <PackageProjectUrl>https://github.com/cntm-labs/nucleus</PackageProjectUrl>
  <RepositoryUrl>https://github.com/cntm-labs/nucleus</RepositoryUrl>
  <PackageTags>authentication;auth;nucleus;sdk</PackageTags>
  <PackageReadmeFile>README.md</PackageReadmeFile>
</PropertyGroup>
```

Same for `Nucleus.AspNetCore.csproj` with `PackageId` = `CntmLabs.Nucleus.AspNetCore`.

**Step 2: Verify**

Run: `cd sdks/dotnet && dotnet restore 2>&1 | tail -1`
Expected: `Restore complete`

**Step 3: Commit**

```bash
git add sdks/dotnet/src/Nucleus/Nucleus.csproj sdks/dotnet/src/Nucleus.AspNetCore/Nucleus.AspNetCore.csproj
git commit -m "chore(sdks): update .NET SDK configs for CntmLabs.Nucleus 0.1.0-dev.1"
```

---

### Task 6: Update Java SDK config

**Files:**
- Modify: `sdks/java/pom.xml`

**Step 1: Update pom.xml**

Change groupId and add publishing metadata:
```xml
<groupId>io.github.cntm-labs</groupId>
<artifactId>nucleus-java</artifactId>
<version>0.1.0-dev.1</version>
<name>nucleus-java</name>
<description>DEV PREVIEW — Not ready for production. Nucleus authentication SDK for Java/Spring Boot.</description>
<url>https://github.com/cntm-labs/nucleus</url>

<licenses>
  <license>
    <name>MIT</name>
    <url>https://opensource.org/licenses/MIT</url>
  </license>
</licenses>

<developers>
  <developer>
    <id>cntm-labs</id>
    <name>cntm-labs</name>
    <url>https://github.com/cntm-labs</url>
  </developer>
</developers>

<scm>
  <connection>scm:git:git://github.com/cntm-labs/nucleus.git</connection>
  <developerConnection>scm:git:ssh://github.com/cntm-labs/nucleus.git</developerConnection>
  <url>https://github.com/cntm-labs/nucleus</url>
</scm>
```

Add Maven Central publishing plugin and GPG signing to build plugins section.

**Step 2: Verify**

Run: `cd sdks/java && mvn validate 2>&1 | tail -1` (if Maven installed) or just verify XML is valid.

**Step 3: Commit**

```bash
git add sdks/java/pom.xml
git commit -m "chore(sdks): update Java SDK config for io.github.cntm-labs 0.1.0-dev.1"
```

---

### Task 7: Update Android SDK configs (2 packages)

**Files:**
- Modify: `sdks/android/build.gradle.kts`
- Modify: `sdks/android-java/build.gradle`

**Step 1: Add publishing config to both files**

For `sdks/android/build.gradle.kts` add:
```kotlin
group = "io.github.cntm-labs"
version = "0.1.0-dev.1"

// Add maven-publish plugin and publishing block
```

For `sdks/android-java/build.gradle` add:
```groovy
group = 'io.github.cntm-labs'
version = '0.1.0-dev.1'
```

**Step 2: Commit**

```bash
git add sdks/android/build.gradle.kts sdks/android-java/build.gradle
git commit -m "chore(sdks): update Android SDK configs for io.github.cntm-labs 0.1.0-dev.1"
```

---

### Task 8: Update Swift SDK config

**Files:**
- Modify: `sdks/swift/Package.swift`
- Create: `sdks/swift/NucleusSwift.podspec`

**Step 1: Update Package.swift**

Ensure package name is correct (already `NucleusSwift`). No version field in Package.swift — versioning via git tags.

**Step 2: Create NucleusSwift.podspec**

```ruby
Pod::Spec.new do |s|
  s.name         = 'NucleusSwift'
  s.version      = '0.1.0-dev.1'
  s.summary      = 'DEV PREVIEW — Not ready for production. Nucleus authentication SDK for iOS/macOS.'
  s.homepage     = 'https://github.com/cntm-labs/nucleus'
  s.license      = { :type => 'MIT', :file => '../../LICENSE' }
  s.author       = { 'cntm-labs' => 'dev@cntm-labs.dev' }
  s.source       = { :git => 'https://github.com/cntm-labs/nucleus.git', :tag => s.version.to_s }
  s.ios.deployment_target = '16.0'
  s.swift_version = '5.9'
  s.source_files = 'Sources/NucleusSwift/**/*.swift'
end
```

**Step 3: Commit**

```bash
git add sdks/swift/Package.swift sdks/swift/NucleusSwift.podspec
git commit -m "chore(sdks): add Swift podspec and update config for 0.1.0-dev.1"
```

---

### Task 9: Update Go SDK config

**Files:**
- Modify: `sdks/go/go.mod`

**Step 1: Update go.mod module path**

Change from:
```
module github.com/nucleus-auth/nucleus-go
```
To:
```
module github.com/cntm-labs/nucleus/sdks/go
```

**Step 2: Commit**

```bash
git add sdks/go/go.mod
git commit -m "chore(sdks): update Go module path to github.com/cntm-labs/nucleus/sdks/go"
```

---

### Task 10: Create README.md for all SDKs

**Files:**
- Create: `sdks/js/README.md`
- Create: `sdks/node/README.md`
- Create: `sdks/react/README.md`
- Create: `sdks/nextjs/README.md`
- Create: `sdks/python/README.md`
- Create: `sdks/rust/README.md`
- Create: `sdks/flutter/README.md`
- Create: `sdks/dotnet/README.md`
- Create: `sdks/java/README.md`
- Create: `sdks/android/README.md`
- Create: `sdks/android-java/README.md`
- Create: `sdks/swift/README.md`
- Create: `sdks/go/README.md`

**Step 1: Create each README with dev warning banner**

Template for all READMEs:
```markdown
# <Package Name>

> **⚠️ DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

Nucleus authentication SDK for <platform>.

## Installation

<install command>

## Quick Start

<minimal code example>

## License

MIT
```

Each README gets platform-specific install command and example:
- npm: `npm i @cntm-labs/node@0.1.0-dev.1`
- Python: `pip install cntm-labs-nucleus==0.1.0.dev1`
- Rust: `cargo add nucleus-rs@0.1.0-dev.1`
- Flutter: `flutter pub add nucleus_flutter:0.1.0-dev.1`
- .NET: `dotnet add package CntmLabs.Nucleus --version 0.1.0-dev.1`
- Java: Maven/Gradle dependency snippet
- Android: Gradle dependency snippet
- Swift: SPM URL + CocoaPods `pod 'NucleusSwift', '~> 0.1.0-dev.1'`
- Go: `go get github.com/cntm-labs/nucleus/sdks/go@v0.1.0-dev.1`

**Step 2: Commit**

```bash
git add sdks/*/README.md
git commit -m "docs(sdks): add README.md with dev preview warning to all SDKs"
```

---

### Task 11: Add runtime dev warnings

**Files:**
- Modify: `sdks/node/src/index.ts` (or main entry)
- Modify: `sdks/nextjs/src/index.ts`
- Modify: `sdks/react/src/index.ts`
- Modify: `sdks/js/src/index.ts`
- Modify: `sdks/python/nucleus/__init__.py` (or main entry)
- Modify: `sdks/rust/src/lib.rs`
- Modify: `sdks/flutter/lib/nucleus_flutter.dart` (or main entry)
- Modify: `sdks/dotnet/src/Nucleus/NucleusClient.cs` (or main entry)
- Modify: `sdks/java/src/main/java/dev/nucleus/NucleusClient.java` (or main entry)
- Modify: `sdks/android/src/main/kotlin/dev/nucleus/Nucleus.kt`
- Modify: `sdks/android-java/src/main/java/dev/nucleus/java/Nucleus.java`
- Modify: `sdks/swift/Sources/NucleusSwift/Nucleus.swift`
- Modify: `sdks/go/nucleus.go` (or main entry)

**Step 1: Add warning to each SDK's initialization**

Each SDK prints on first init:
```
[Nucleus] WARNING: You are using a dev preview (0.1.0-dev.1). Do not use in production.
```

Examples per language:

**TypeScript (all 4 npm packages):**
```typescript
const VERSION = '0.1.0-dev.1';
if (VERSION.includes('-dev')) {
  console.warn(`[Nucleus] WARNING: You are using a dev preview (${VERSION}). Do not use in production.`);
}
```

**Python:**
```python
import warnings
__version__ = "0.1.0.dev1"
if "dev" in __version__:
    warnings.warn(f"[Nucleus] You are using a dev preview ({__version__}). Do not use in production.", stacklevel=2)
```

**Rust:**
```rust
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn init() {
    if VERSION.contains("dev") {
        eprintln!("[Nucleus] WARNING: You are using a dev preview ({VERSION}). Do not use in production.");
    }
}
```

**Dart (Flutter):**
```dart
const nucleusVersion = '0.1.0-dev.1';

void _printDevWarning() {
  if (nucleusVersion.contains('dev')) {
    debugPrint('[Nucleus] WARNING: You are using a dev preview ($nucleusVersion). Do not use in production.');
  }
}
```

**C# (.NET):**
```csharp
internal static class NucleusWarning {
    private static bool _warned;
    internal static void PrintIfDev() {
        if (_warned) return;
        var version = typeof(NucleusWarning).Assembly.GetName().Version?.ToString() ?? "unknown";
        if (version.Contains("dev")) {
            Console.Error.WriteLine($"[Nucleus] WARNING: You are using a dev preview ({version}). Do not use in production.");
        }
        _warned = true;
    }
}
```

**Java:**
```java
private static boolean warned = false;
static void printDevWarning() {
    if (!warned) {
        String version = "0.1.0-dev.1";
        if (version.contains("dev")) {
            System.err.println("[Nucleus] WARNING: You are using a dev preview (" + version + "). Do not use in production.");
        }
        warned = true;
    }
}
```

**Swift:**
```swift
private static var warned = false
static func printDevWarning() {
    guard !warned else { return }
    let version = "0.1.0-dev.1"
    if version.contains("dev") {
        print("[Nucleus] WARNING: You are using a dev preview (\(version)). Do not use in production.")
    }
    warned = true
}
```

**Go:**
```go
const Version = "0.1.0-dev.1"

var warned bool

func init() {
	if !warned && strings.Contains(Version, "dev") {
		fmt.Fprintf(os.Stderr, "[Nucleus] WARNING: You are using a dev preview (%s). Do not use in production.\n", Version)
		warned = true
	}
}
```

**Kotlin (Android):**
```kotlin
companion object {
    private var warned = false
    internal fun printDevWarning() {
        if (!warned) {
            val version = "0.1.0-dev.1"
            if ("dev" in version) {
                android.util.Log.w("Nucleus", "WARNING: You are using a dev preview ($version). Do not use in production.")
            }
            warned = true
        }
    }
}
```

**Step 2: Commit**

```bash
git add sdks/
git commit -m "feat(sdks): add runtime dev preview warnings to all SDKs"
```

---

### Task 12: Create publish-sdks.yml workflow

**Files:**
- Create: `.github/workflows/publish-sdks.yml`

**Step 1: Create the workflow**

```yaml
name: Publish SDKs

on:
  workflow_dispatch:
    inputs:
      sdk:
        description: 'Which SDK to publish'
        required: true
        type: choice
        options:
          - all
          - npm
          - python
          - rust
          - flutter
          - nuget
          - maven
          - swift
          - go
      version:
        description: 'Version to publish (e.g. 0.1.0-dev.2)'
        required: true
        default: '0.1.0-dev.1'
      dry-run:
        description: 'Dry run (do not actually publish)'
        required: false
        type: boolean
        default: true

jobs:
  # --- npm (4 packages) ---
  npm:
    if: inputs.sdk == 'all' || inputs.sdk == 'npm'
    runs-on: ubuntu-latest
    strategy:
      matrix:
        package: [js, node, react, nextjs]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: '22', registry-url: 'https://registry.npmjs.org' }
      - name: Update version
        run: cd sdks/${{ matrix.package }} && npm version ${{ inputs.version }} --no-git-tag-version --allow-same-version
      - name: Build
        run: cd sdks/${{ matrix.package }} && npm install && npm run build --if-present
      - name: Publish
        if: inputs.dry-run == false
        run: cd sdks/${{ matrix.package }} && npm publish --access public
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
      - name: Dry run
        if: inputs.dry-run == true
        run: cd sdks/${{ matrix.package }} && npm publish --access public --dry-run
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}

  # --- Python ---
  python:
    if: inputs.sdk == 'all' || inputs.sdk == 'python'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with: { python-version: '3.12' }
      - name: Install build tools
        run: pip install build twine
      - name: Build
        run: cd sdks/python && python -m build
      - name: Publish
        if: inputs.dry-run == false
        run: cd sdks/python && twine upload dist/*
        env:
          TWINE_USERNAME: __token__
          TWINE_PASSWORD: ${{ secrets.PYPI_TOKEN }}
      - name: Dry run
        if: inputs.dry-run == true
        run: cd sdks/python && twine check dist/*

  # --- Rust ---
  rust:
    if: inputs.sdk == 'all' || inputs.sdk == 'rust'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Publish
        if: inputs.dry-run == false
        run: cd sdks/rust && cargo publish --token ${{ secrets.CRATES_IO_TOKEN }}
      - name: Dry run
        if: inputs.dry-run == true
        run: cd sdks/rust && cargo publish --dry-run

  # --- Flutter ---
  flutter:
    if: inputs.sdk == 'all' || inputs.sdk == 'flutter'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: subosito/flutter-action@v2
        with: { flutter-version: '3.24.0' }
      - name: Setup pub credentials
        run: |
          mkdir -p $HOME/.config/dart
          echo '${{ secrets.PUB_DEV_CREDENTIALS }}' > $HOME/.config/dart/pub-credentials.json
      - name: Publish
        if: inputs.dry-run == false
        run: cd sdks/flutter && dart pub publish --force
      - name: Dry run
        if: inputs.dry-run == true
        run: cd sdks/flutter && dart pub publish --dry-run

  # --- NuGet (.NET) ---
  nuget:
    if: inputs.sdk == 'all' || inputs.sdk == 'nuget'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-dotnet@v4
        with: { dotnet-version: '8.0' }
      - name: Pack
        run: |
          cd sdks/dotnet/src/Nucleus && dotnet pack -c Release -o ../../../../nupkgs
          cd ../Nucleus.AspNetCore && dotnet pack -c Release -o ../../../../nupkgs
      - name: Publish
        if: inputs.dry-run == false
        run: dotnet nuget push "nupkgs/*.nupkg" --api-key ${{ secrets.NUGET_API_KEY }} --source https://api.nuget.org/v3/index.json
      - name: Dry run
        if: inputs.dry-run == true
        run: ls -la nupkgs/

  # --- Maven Central (Java + Android) ---
  maven:
    if: inputs.sdk == 'all' || inputs.sdk == 'maven'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-java@v4
        with: { distribution: 'temurin', java-version: '17' }
      - name: Import GPG key
        run: echo "${{ secrets.MAVEN_GPG_KEY }}" | base64 -d | gpg --batch --import
      - name: Deploy Java SDK
        if: inputs.dry-run == false
        run: |
          cd sdks/java && mvn deploy -P release \
            -Dgpg.passphrase="${{ secrets.MAVEN_GPG_PASSPHRASE }}"
        env:
          MAVEN_USERNAME: ${{ secrets.MAVEN_USERNAME }}
          MAVEN_PASSWORD: ${{ secrets.MAVEN_PASSWORD }}
      - name: Dry run
        if: inputs.dry-run == true
        run: cd sdks/java && mvn package -P release -DskipTests

  # --- Swift (CocoaPods) ---
  swift:
    if: inputs.sdk == 'all' || inputs.sdk == 'swift'
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - name: Validate podspec
        run: cd sdks/swift && pod lib lint NucleusSwift.podspec --allow-warnings
      - name: Publish
        if: inputs.dry-run == false
        run: cd sdks/swift && pod trunk push NucleusSwift.podspec --allow-warnings
        env:
          COCOAPODS_TRUNK_TOKEN: ${{ secrets.COCOAPODS_TRUNK_TOKEN }}
      - name: Dry run
        if: inputs.dry-run == true
        run: echo "Dry run — podspec validated"

  # --- Go ---
  go:
    if: inputs.sdk == 'all' || inputs.sdk == 'go'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-go@v5
        with: { go-version: '1.21' }
      - name: Verify module
        run: cd sdks/go && go vet ./...
      - name: Create tag
        if: inputs.dry-run == false
        run: |
          git tag "sdks/go/v${{ inputs.version }}"
          git push origin "sdks/go/v${{ inputs.version }}"
      - name: Dry run
        if: inputs.dry-run == true
        run: echo "Would create tag sdks/go/v${{ inputs.version }}"
```

**Step 2: Verify YAML is valid**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/publish-sdks.yml')); print('OK')"`
Expected: `OK`

**Step 3: Commit**

```bash
git add .github/workflows/publish-sdks.yml
git commit -m "ci: add publish-sdks.yml workflow for all 13 SDKs"
```

---

### Task 13: Push and run first dry-run publish

**Step 1: Push all changes**

```bash
git push origin main
```

**Step 2: Trigger dry-run**

Go to: `https://github.com/cntm-labs/nucleus/actions/workflows/publish-sdks.yml`
Click "Run workflow" with:
- sdk: `all`
- version: `0.1.0-dev.1`
- dry-run: `true` (checked)

**Step 3: Fix any issues from dry-run**

Review each job's output. Common issues:
- Missing build scripts in package.json
- Invalid metadata fields
- Missing files referenced in configs

**Step 4: Run actual publish**

Once dry-run passes, re-run with `dry-run: false`.

**Step 5: Verify packages are live**

Check each registry:
- https://www.npmjs.com/package/@cntm-labs/node
- https://pypi.org/project/cntm-labs-nucleus/
- https://crates.io/crates/nucleus-rs
- https://pub.dev/packages/nucleus_flutter
- https://www.nuget.org/packages/CntmLabs.Nucleus
- https://central.sonatype.com/artifact/io.github.cntm-labs/nucleus-java
- https://cocoapods.org/pods/NucleusSwift
- https://pkg.go.dev/github.com/cntm-labs/nucleus/sdks/go

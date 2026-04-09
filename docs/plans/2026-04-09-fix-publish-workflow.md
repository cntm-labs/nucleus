# Fix Publish Workflow — Complete 12 Failed Registry Publishes

> **For Claude:** Execute this plan in a new session. Be thorough — verify every step on the actual registry, not just GitHub Actions.

**Problem:** 12 out of 15 packages were NOT published to their registries despite GitHub releases being created. Only 3 succeeded (cntm-nucleus on crates.io, cntm-nucleus on PyPI, cntm_nucleus on pub.dev).

**Root cause:** The sdk-publish.yml workflow had bugs (empty version string, old package names) and release-please created v0.2.0 releases before v0.1.0 existed on some registries.

## Task 0: Clean Up Orphan Tags and Failed Releases (DO THIS FIRST)

### Orphan/broken tags to delete
```
sdks/go/v       ← broken empty tag (Go publish with empty version)
sdks/go/v0.2.0  ← orphan tag (no matching release, wrong format)
```

### Duplicate release to clean
```
cntm-nucleus-android-java-v0.2.0  ← superseded by v0.2.1, delete this release + tag
```

### Steps
```bash
# Delete broken tags
gh api -X DELETE repos/cntm-labs/nucleus/git/refs/tags/sdks/go/v
gh api -X DELETE repos/cntm-labs/nucleus/git/refs/tags/sdks/go/v0.2.0

# Delete superseded release + tag
gh release delete cntm-nucleus-android-java-v0.2.0 --yes
gh api -X DELETE repos/cntm-labs/nucleus/git/refs/tags/cntm-nucleus-android-java-v0.2.0
```

### After cleanup, expected state
- **10 tags** (one per successfully created release)
- **9 releases** (removed the duplicate v0.2.0)
- **0 stale branches** (already cleaned)

---

## Current State (as of 2026-04-09)

### Published ✅
| Package | Registry | Version |
|---------|----------|---------|
| `cntm-nucleus` | crates.io | 0.2.0 |
| `cntm-nucleus` | PyPI | 0.1.0 |
| `cntm_nucleus` | pub.dev | (exists) |

### NOT Published ❌
| Package | Registry | GitHub Release Tag |
|---------|----------|--------------------|
| `cntm-nucleus-server` | crates.io | cntm-nucleus-server-v0.2.0 (no release PR merged) |
| `@cntm-labs/nucleus-js` | npm | @cntm-labs/nucleus-js-v0.2.0 |
| `@cntm-labs/nucleus-node` | npm | (no release PR merged) |
| `@cntm-labs/nucleus-react` | npm | @cntm-labs/nucleus-react-v0.2.0 |
| `@cntm-labs/nucleus-nextjs` | npm | (no release PR merged) |
| `@cntm-labs/nucleus-observation` | npm | (no release PR merged) |
| `Cntm.Nucleus` | NuGet | Cntm.Nucleus-v0.2.0 |
| `io.github.cntm-labs:nucleus` | Maven | cntm-nucleus-java-v0.2.0 |
| `io.github.cntm-labs:nucleus-android` | Maven | cntm-nucleus-android-v0.2.0 |
| `io.github.cntm-labs:nucleus-java` | Maven | cntm-nucleus-android-java-v0.2.0/v0.2.1 |
| `CntmNucleus` | CocoaPods | CntmNucleus-v0.2.0 (no release PR merged) |
| Go module | pkg.go.dev | cntm-nucleus-go-v0.2.0 |

### Secrets Available (all configured)
NPM_TOKEN, PYPI_TOKEN, CRATES_IO_TOKEN, PUB_DEV_CREDENTIALS, NUGET_API_KEY, MAVEN_USERNAME, MAVEN_PASSWORD, MAVEN_GPG_KEY, MAVEN_GPG_PASSPHRASE, COCOAPODS_TRUNK_TOKEN

### Stale Branches
All cleaned up. Only `main` exists.

---

## Task 1: Analyze Publish Failure Logs

### Step 1: Get the publish workflow run IDs that failed
```bash
gh run list --workflow=release-please.yml --limit 20 --json databaseId,conclusion,createdAt \
  --jq '.[] | select(.conclusion == "failure") | "\(.databaseId) \(.createdAt)"'
```

### Step 2: For each failed run, extract the publish job logs
```bash
gh run view <RUN_ID> --log-failed 2>&1 | head -100
```

### Step 3: Document exact error for each registry
Create a table: registry → error message → root cause → fix

### Step 4: Verify — check which publish jobs succeeded/failed per registry
Check the actual publish steps in each workflow run.

---

## Task 2: Fix sdk-publish.yml Workflow

Based on the failure analysis, fix these known issues:

### Step 1: Fix version passing
The release-please.yml currently hardcodes `version: '0.2.0'`. This needs to dynamically extract the version from the release output. Use the release-please action outputs.

### Step 2: Fix publish verification
Add a real verification step that checks each registry API after publishing:
```yaml
- name: Verify npm publish
  run: |
    sleep 30
    npm view "@cntm-labs/nucleus-${{ matrix.package }}" version || exit 1
```

### Step 3: Fix crates.io server publish
The `rust` job in sdk-publish.yml only publishes the SDK (`sdks/rust`), not the server (`crates/nucleus-server`). Add a separate job or modify to handle both.

### Step 4: Fix podspec file name
Verify `CntmNucleus.podspec` is correctly referenced in the Swift publish step.

### Step 5: Make publish idempotent
Some registries reject re-publishing the same version. Add `|| true` or version-exists checks.

### Step 6: Test with dry_run first
Before real publish, run the workflow with `dry_run: true` for every SDK to catch errors.

---

## Task 3: Re-publish the 12 Failed Packages

### Step 1: Trigger sdk-publish.yml manually for each failed registry
Use `gh workflow run` with specific SDK and version:
```bash
# npm
gh workflow run sdk-publish.yml -f sdk=npm -f version=0.2.0 -f dry_run=false

# rust (server)
gh workflow run sdk-publish.yml -f sdk=rust -f version=0.2.0 -f dry_run=false

# nuget
gh workflow run sdk-publish.yml -f sdk=nuget -f version=0.2.0 -f dry_run=false

# maven
gh workflow run sdk-publish.yml -f sdk=maven -f version=0.2.0 -f dry_run=false

# swift
gh workflow run sdk-publish.yml -f sdk=swift -f version=0.2.0 -f dry_run=false

# go
gh workflow run sdk-publish.yml -f sdk=go -f version=0.2.0 -f dry_run=false
```

### Step 2: After each publish, verify on the ACTUAL registry
Do NOT trust GitHub Actions status. Check the real URL:
```bash
# npm
curl -s "https://registry.npmjs.org/@cntm-labs/nucleus-react" | python3 -c "import sys,json; print(json.load(sys.stdin)['dist-tags']['latest'])"

# crates.io
curl -s "https://crates.io/api/v1/crates/cntm-nucleus-server" | python3 -c "import sys,json; print(json.load(sys.stdin)['crate']['newest_version'])"

# NuGet
curl -s "https://api.nuget.org/v3-flatcontainer/cntm.nucleus/index.json" | python3 -c "import sys,json; print(json.load(sys.stdin)['versions'])"

# Maven
curl -s "https://search.maven.org/solrsearch/select?q=g:io.github.cntm-labs+AND+a:nucleus&rows=1"

# CocoaPods
curl -s "https://trunk.cocoapods.org/api/v1/pods/CntmNucleus"

# Go
curl -s "https://proxy.golang.org/github.com/cntm-labs/nucleus/sdks/go/@v/list"
```

### Step 3: Handle version conflicts
If a registry already has v0.2.0 (from partial publish), we may need v0.2.1. Check before publishing.

---

## Task 4: Merge Remaining Release PRs (6 packages)

These 6 packages never had their release PRs merged (they were closed due to conflicts):
1. CntmNucleus (Swift)
2. cntm-nucleus-python
3. cntm-nucleus-server
4. @cntm-labs/nucleus-observation
5. @cntm-labs/nucleus-node
6. @cntm-labs/nucleus-nextjs

### Step 1: Trigger release-please to create fresh PRs
Push empty commit or wait for release-please to create new PRs.

### Step 2: Merge one at a time
After each merge, WAIT for release-please to rebase remaining PRs before merging the next.

### Step 3: Verify publish after each merge
Check the actual registry, not just GitHub.

---

## Task 5: Final Verification

### Step 1: Check ALL 15 registries
Run the full registry check script and confirm every package exists with correct version.

### Step 2: Test install from each registry
```bash
# npm
npm info @cntm-labs/nucleus-react
# cargo
cargo search cntm-nucleus
# pip
pip index versions cntm-nucleus
# etc.
```

### Step 3: Update memory with final publish status

---

## Verification Checklist

After ALL tasks complete, every row must be ✅:

| # | Package | Registry | Version | Verified |
|---|---------|----------|---------|----------|
| 1 | `cntm-nucleus-server` | crates.io | 0.2.0 | ☐ |
| 2 | `cntm-nucleus` | crates.io | 0.2.0 | ✅ (already) |
| 3 | `@cntm-labs/nucleus-js` | npm | 0.2.0 | ☐ |
| 4 | `@cntm-labs/nucleus-node` | npm | 0.2.0 | ☐ |
| 5 | `@cntm-labs/nucleus-react` | npm | 0.2.0 | ☐ |
| 6 | `@cntm-labs/nucleus-nextjs` | npm | 0.2.0 | ☐ |
| 7 | `@cntm-labs/nucleus-observation` | npm | 0.2.0 | ☐ |
| 8 | `cntm-nucleus` | PyPI | 0.2.0 | ☐ (has 0.1.0, needs 0.2.0) |
| 9 | `cntm_nucleus` | pub.dev | 0.2.0 | ☐ |
| 10 | `Cntm.Nucleus` | NuGet | 0.2.0 | ☐ |
| 11 | `io.github.cntm-labs:nucleus` | Maven | 0.2.0 | ☐ |
| 12 | `io.github.cntm-labs:nucleus-android` | Maven | 0.2.0 | ☐ |
| 13 | `io.github.cntm-labs:nucleus-java` | Maven | 0.2.0 | ☐ |
| 14 | `CntmNucleus` | CocoaPods | 0.2.0 | ☐ |
| 15 | Go module | pkg.go.dev | 0.2.0 | ☐ |

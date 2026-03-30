# Phase 4: Test Coverage — Critical Path Tests

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add tests for the most critical untested code paths: Rust auth handler integration, DB repository SQL, server SDK token verification (Python, Go, Java, .NET, Rust SDK), and React/Next.js component rendering.

**Architecture:** Each task targets one layer or SDK. Backend handler tests use the existing mock pattern (inline `#[cfg(test)]` modules with `MockUserRepo` etc.). Server SDK tests mock the JWKS HTTP endpoint with a local test server. React/Next.js component tests use vitest + @testing-library/react. We do NOT add integration tests requiring a real Postgres/Redis — that's a separate infrastructure task.

**Tech Stack:** Rust (`#[tokio::test]`, mock traits), Python (pytest, respx), Go (testing, httptest), Java (JUnit 5, WireMock), .NET (xUnit, MockHttpMessageHandler), Rust SDK (tokio::test, mockito), React/Next.js (vitest, @testing-library/react)

**Execution order:** Rust handlers → Rust DB repos → Python → Go → Java → .NET → Rust SDK → React components → Next.js components

---

## Task 1: Rust Auth Handler Tests (sign_in, sign_up)

### Problem
`handlers/sign_in.rs` (59 lines) and `handlers/sign_up.rs` (66 lines) parse JSON requests, call `AuthService`, and return HTTP responses — zero tests.

### Files
- Modify: `crates/nucleus-auth/src/handlers/sign_in.rs` — add `#[cfg(test)]` module
- Modify: `crates/nucleus-auth/src/handlers/sign_up.rs` — add `#[cfg(test)]` module

### Step 1: Add tests to sign_in.rs

Add at the bottom of `crates/nucleus-auth/src/handlers/sign_in.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::tests::{make_service, make_test_user, make_password_credential, make_session_service};
    use axum::extract::State;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_sign_in_success() {
        let (auth_service, _user_repo, _cred_repo, _audit_repo) = make_service();
        let session_service = make_session_service();

        // Pre-populate user + credential via service
        let user = make_test_user("test@example.com");
        let credential = make_password_credential(&user.id, "password123");
        // Insert into mock repos...

        let req = SignInRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            project_id: None,
        };

        let result = handle_sign_in(
            State(auth_service),
            Json(req),
        ).await;

        assert!(result.is_ok());
        let (status, json) = result.unwrap();
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_sign_in_invalid_email() {
        let (auth_service, _, _, _) = make_service();

        let req = SignInRequest {
            email: "nonexistent@example.com".to_string(),
            password: "password123".to_string(),
            project_id: None,
        };

        let result = handle_sign_in(State(auth_service), Json(req)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sign_in_wrong_password() {
        let (auth_service, _, _, _) = make_service();

        let req = SignInRequest {
            email: "test@example.com".to_string(),
            password: "wrongpassword".to_string(),
            project_id: None,
        };

        let result = handle_sign_in(State(auth_service), Json(req)).await;
        assert!(result.is_err());
    }
}
```

### Step 2: Add tests to sign_up.rs

Similar pattern — test success case (returns `CREATED`), duplicate email case, invalid email case, weak password case.

### Step 3: Verify

Run: `cargo test -p nucleus-auth -- handlers::sign_in::tests`
Run: `cargo test -p nucleus-auth -- handlers::sign_up::tests`

### Step 4: Commit

```bash
git commit -m "test(auth): add handler tests for sign_in and sign_up"
```

---

## Task 2: Rust Server Handler Integration Tests

### Problem
`nucleus-server/src/handlers/auth.rs` (268 lines) has 13 handler functions that extract `AppState`, delegate to `nucleus-auth`, and map errors — zero tests.

### Files
- Modify: `crates/nucleus-server/src/handlers/auth.rs` — add `#[cfg(test)]` module

### Step 1: Add integration-style tests

Test the server handler wrappers that compose AppState → delegate → return response:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;

    async fn test_app_state() -> Arc<AppState> {
        // Build AppState with mock repos (reusing existing mocks from nucleus-auth)
        // This validates the full composition chain
        todo!("Build test AppState with mocks")
    }

    #[tokio::test]
    async fn test_handle_sign_up_returns_201() {
        let state = test_app_state().await;
        let req = Json(serde_json::json!({
            "email": "new@example.com",
            "password": "password123",
        }));
        let result = handle_sign_up(State(state), req).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_sign_in_returns_200() {
        let state = test_app_state().await;
        // Pre-populate user...
        let req = Json(serde_json::json!({
            "email": "test@example.com",
            "password": "password123",
        }));
        let result = handle_sign_in(State(state), req).await;
        assert!(result.is_ok());
    }
}
```

### Step 2: Test error cases

- Sign-in with non-existent user → expect auth error
- Sign-up with duplicate email → expect conflict error
- Sign-out without valid JWT → expect unauthorized

### Step 3: Verify and commit

Run: `cargo test -p nucleus-server -- handlers::auth::tests`

```bash
git commit -m "test(server): add integration tests for auth handler wrappers"
```

---

## Task 3: Rust DB Repository Mock Tests

### Problem
All 9 repository implementations in `nucleus-db` have zero tests. The repos execute SQL queries that could silently fail. Without a test database, we test the mock implementations used by services.

### Files
- Modify: `crates/nucleus-db/src/repos/user_repo.rs` — add mock validation tests
- Modify: `crates/nucleus-db/src/repos/session_repo.rs` — add mock tests

### Step 1: Add UserRepository mock completeness tests

At the bottom of `user_repo.rs`, add tests that validate the mock implementations match expected behavior:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockUserRepo {
        users: Mutex<Vec<User>>,
    }

    // ... implement UserRepository for MockUserRepo ...

    #[tokio::test]
    async fn test_create_user_returns_user_with_id() {
        let repo = MockUserRepo { users: Mutex::new(vec![]) };
        let project_id = ProjectId::from(Uuid::new_v4());
        let new_user = NewUser {
            email: "test@example.com".to_string(),
            username: None, first_name: Some("Test".to_string()),
            last_name: None, external_id: None, phone: None,
            avatar_url: None, metadata: None,
        };
        let user = repo.create(&project_id, &new_user).await.unwrap();
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.first_name, Some("Test".to_string()));
        assert!(!user.id.as_ref().is_nil());
    }

    #[tokio::test]
    async fn test_find_by_email_returns_none_for_nonexistent() {
        let repo = MockUserRepo { users: Mutex::new(vec![]) };
        let project_id = ProjectId::from(Uuid::new_v4());
        let result = repo.find_by_email(&project_id, "nobody@example.com").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_soft_delete_excludes_from_find() {
        let repo = MockUserRepo { users: Mutex::new(vec![]) };
        let project_id = ProjectId::from(Uuid::new_v4());
        let new_user = NewUser { email: "delete@example.com".to_string(), ..Default::default() };
        let user = repo.create(&project_id, &new_user).await.unwrap();

        repo.soft_delete(&project_id, &user.id).await.unwrap();
        let found = repo.find_by_id(&project_id, &user.id).await.unwrap();
        assert!(found.is_none()); // Soft-deleted users should not be found
    }
}
```

### Step 2: Add SessionRepository Redis mock tests

Test session create → validate → revoke → validate-fails flow.

### Step 3: Verify and commit

Run: `cargo test -p nucleus-db`

```bash
git commit -m "test(db): add mock repository tests for UserRepo and SessionRepo"
```

---

## Task 4: Python SDK Tests

### Problem
Python SDK has 0 tests. `verify_token()` and `NucleusClient` are the critical paths.

### Files
- Create: `sdks/python/tests/__init__.py`
- Create: `sdks/python/tests/test_verify.py`
- Create: `sdks/python/tests/test_client.py`
- Modify: `sdks/python/pyproject.toml` — add test dependencies

### Step 1: Add test dependencies

In `pyproject.toml`, add:

```toml
[project.optional-dependencies]
test = ["pytest>=8.0", "pytest-asyncio>=0.24", "respx>=0.22", "cryptography>=43.0"]
```

### Step 2: Create test_verify.py

```python
# sdks/python/tests/test_verify.py
import pytest
import jwt
from datetime import datetime, timezone, timedelta
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.hazmat.primitives import serialization
from nucleus.verify import verify_token
from nucleus.claims import NucleusClaims

# Generate test RSA key pair
_private_key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
_public_key = _private_key.public_key()

def _make_token(claims: dict, kid: str = "test-key-1") -> str:
    return jwt.encode(claims, _private_key, algorithm="RS256", headers={"kid": kid})

def _valid_claims() -> dict:
    now = datetime.now(timezone.utc)
    return {
        "sub": "user_123",
        "iss": "https://api.test.com",
        "aud": "project_456",
        "exp": int((now + timedelta(hours=1)).timestamp()),
        "iat": int(now.timestamp()),
        "jti": "jwt_abc",
    }

class TestVerifyToken:
    def test_valid_token_returns_claims(self, mock_jwks_server):
        token = _make_token(_valid_claims())
        claims = verify_token(token, jwks_url=mock_jwks_server.url)
        assert isinstance(claims, NucleusClaims)
        assert claims.sub == "user_123"

    def test_expired_token_raises(self, mock_jwks_server):
        claims = _valid_claims()
        claims["exp"] = int((datetime.now(timezone.utc) - timedelta(hours=1)).timestamp())
        token = _make_token(claims)
        with pytest.raises(Exception):
            verify_token(token, jwks_url=mock_jwks_server.url)

    def test_invalid_signature_raises(self, mock_jwks_server):
        other_key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
        token = jwt.encode(_valid_claims(), other_key, algorithm="RS256", headers={"kid": "test-key-1"})
        with pytest.raises(Exception):
            verify_token(token, jwks_url=mock_jwks_server.url)
```

### Step 3: Create test_client.py

Test `NucleusClient` initialization, `verify_token()` delegation, resource cleanup.

### Step 4: Verify

```bash
cd sdks/python && pip install -e ".[test]" && pytest tests/ -v
```

### Step 5: Commit

```bash
git commit -m "test(python): add token verification and client tests"
```

---

## Task 5: Go SDK Tests

### Problem
Go SDK has 0 tests despite Go's built-in testing.

### Files
- Create: `sdks/go/verify_test.go`
- Create: `sdks/go/middleware_test.go`
- Create: `sdks/go/client_test.go`

### Step 1: Create verify_test.go

```go
// sdks/go/verify_test.go
package nucleus

import (
    "crypto/rand"
    "crypto/rsa"
    "encoding/json"
    "net/http"
    "net/http/httptest"
    "testing"
    "time"

    "github.com/golang-jwt/jwt/v5"
)

func generateTestKey(t *testing.T) *rsa.PrivateKey {
    key, err := rsa.GenerateKey(rand.Reader, 2048)
    if err != nil { t.Fatal(err) }
    return key
}

func makeToken(t *testing.T, key *rsa.PrivateKey, claims jwt.MapClaims) string {
    token := jwt.NewWithClaims(jwt.SigningMethodRS256, claims)
    token.Header["kid"] = "test-key-1"
    signed, err := token.SignedString(key)
    if err != nil { t.Fatal(err) }
    return signed
}

func startJWKSServer(t *testing.T, key *rsa.PrivateKey) *httptest.Server {
    // Serve JWKS endpoint with the test public key
    return httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        // Return JWKS JSON with the public key
        jwks := buildJWKS(key)
        w.Header().Set("Content-Type", "application/json")
        json.NewEncoder(w).Encode(jwks)
    }))
}

func TestVerifyToken_Valid(t *testing.T) {
    key := generateTestKey(t)
    server := startJWKSServer(t, key)
    defer server.Close()

    token := makeToken(t, key, jwt.MapClaims{
        "sub": "user_123",
        "exp": time.Now().Add(time.Hour).Unix(),
        "iat": time.Now().Unix(),
    })

    claims, err := VerifyToken(token, server.URL)
    if err != nil { t.Fatalf("expected no error, got %v", err) }
    if claims.Subject != "user_123" { t.Errorf("expected user_123, got %s", claims.Subject) }
}

func TestVerifyToken_Expired(t *testing.T) {
    key := generateTestKey(t)
    server := startJWKSServer(t, key)
    defer server.Close()

    token := makeToken(t, key, jwt.MapClaims{
        "sub": "user_123",
        "exp": time.Now().Add(-time.Hour).Unix(),
    })

    _, err := VerifyToken(token, server.URL)
    if err == nil { t.Fatal("expected error for expired token") }
}

func TestVerifyToken_WrongKey(t *testing.T) {
    key := generateTestKey(t)
    wrongKey := generateTestKey(t)
    server := startJWKSServer(t, key) // Server has key1's public key
    defer server.Close()

    token := makeToken(t, wrongKey, jwt.MapClaims{ // Signed with key2
        "sub": "user_123",
        "exp": time.Now().Add(time.Hour).Unix(),
    })

    _, err := VerifyToken(token, server.URL)
    if err == nil { t.Fatal("expected error for wrong key") }
}
```

### Step 2: Create middleware_test.go

```go
func TestProtect_ValidToken(t *testing.T) {
    key := generateTestKey(t)
    server := startJWKSServer(t, key)
    defer server.Close()

    client := &NucleusClient{config: Config{SecretKey: "sk_test"}}
    // Override JWKS URL for test...

    handler := client.Protect()(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        claims := ClaimsFromContext(r.Context())
        if claims == nil { t.Fatal("expected claims in context") }
        w.WriteHeader(http.StatusOK)
    }))

    token := makeToken(t, key, jwt.MapClaims{
        "sub": "user_123",
        "exp": time.Now().Add(time.Hour).Unix(),
    })

    req := httptest.NewRequest("GET", "/", nil)
    req.Header.Set("Authorization", "Bearer "+token)
    rr := httptest.NewRecorder()
    handler.ServeHTTP(rr, req)

    if rr.Code != http.StatusOK { t.Errorf("expected 200, got %d", rr.Code) }
}

func TestProtect_MissingToken(t *testing.T) {
    client := &NucleusClient{config: Config{SecretKey: "sk_test"}}
    handler := client.Protect()(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        t.Fatal("handler should not be called")
    }))

    req := httptest.NewRequest("GET", "/", nil)
    rr := httptest.NewRecorder()
    handler.ServeHTTP(rr, req)

    if rr.Code != http.StatusUnauthorized { t.Errorf("expected 401, got %d", rr.Code) }
}
```

### Step 3: Verify

```bash
cd sdks/go && go test ./... -v
```

### Step 4: Commit

```bash
git commit -m "test(go): add token verification and middleware tests"
```

---

## Task 6: Java SDK Tests

### Problem
Java SDK has 0 tests. `NucleusTokenVerifier.verify()` is the critical path.

### Files
- Create: `sdks/java/src/test/java/dev/nucleus/NucleusTokenVerifierTest.java`
- Create: `sdks/java/src/test/java/dev/nucleus/NucleusClientTest.java`
- Modify: `sdks/java/pom.xml` — add test dependencies

### Step 1: Add test dependencies to pom.xml

```xml
<dependency>
    <groupId>org.junit.jupiter</groupId>
    <artifactId>junit-jupiter</artifactId>
    <version>5.11.4</version>
    <scope>test</scope>
</dependency>
<dependency>
    <groupId>com.github.tomakehurst</groupId>
    <artifactId>wiremock-jre8</artifactId>
    <version>3.0.1</version>
    <scope>test</scope>
</dependency>
```

### Step 2: Create NucleusTokenVerifierTest.java

```java
package dev.nucleus;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class NucleusTokenVerifierTest {
    @Test
    void verifyValidToken() {
        // Generate RSA key pair, create JWKS mock, sign token, verify
    }

    @Test
    void rejectExpiredToken() {
        // Create token with past exp, verify throws
    }

    @Test
    void rejectWrongSignature() {
        // Sign with key A, serve key B in JWKS, verify throws
    }
}
```

### Step 3: Create NucleusClientTest.java

Test builder pattern, default base URL, `users()` and `organizations()` factories.

### Step 4: Verify

```bash
cd sdks/java && mvn test
```

### Step 5: Commit

```bash
git commit -m "test(java): add token verification and client builder tests"
```

---

## Task 7: .NET SDK Tests

### Problem
.NET SDK has 0 tests. Token verification with `ConfigurationManager<OpenIdConnectConfiguration>` is critical.

### Files
- Create: `sdks/dotnet/tests/Nucleus.Tests/Nucleus.Tests.csproj`
- Create: `sdks/dotnet/tests/Nucleus.Tests/TokenVerifierTests.cs`
- Create: `sdks/dotnet/tests/Nucleus.Tests/ClientTests.cs`

### Step 1: Create test project

```xml
<!-- sdks/dotnet/tests/Nucleus.Tests/Nucleus.Tests.csproj -->
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
    <IsPackable>false</IsPackable>
  </PropertyGroup>
  <ItemGroup>
    <PackageReference Include="Microsoft.NET.Test.Sdk" Version="17.11.1" />
    <PackageReference Include="xunit" Version="2.9.2" />
    <PackageReference Include="xunit.runner.visualstudio" Version="2.8.2" />
    <PackageReference Include="Moq" Version="4.20.72" />
    <ProjectReference Include="../../src/Nucleus/Nucleus.csproj" />
  </ItemGroup>
</Project>
```

### Step 2: Create TokenVerifierTests.cs

Test valid token, expired token, wrong key, claim mapping for all types.

### Step 3: Add test project to solution

```bash
cd sdks/dotnet && dotnet sln add tests/Nucleus.Tests/Nucleus.Tests.csproj
```

### Step 4: Verify

```bash
cd sdks/dotnet && dotnet test
```

### Step 5: Commit

```bash
git commit -m "test(dotnet): add token verification and claim mapping tests"
```

---

## Task 8: Rust SDK Tests

### Problem
Rust SDK has 0 tests. `JwksVerifier` with `RwLock` caching is the critical path.

### Files
- Modify: `sdks/rust/src/verify.rs` — add `#[cfg(test)]` module

### Step 1: Add tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_verifier_creation() {
        let verifier = JwksVerifier::new("https://api.test.com/.well-known/jwks.json", 3600);
        assert!(verifier.jwks_url.contains("jwks.json"));
    }

    #[tokio::test]
    async fn test_verify_invalid_token_returns_error() {
        let verifier = JwksVerifier::new("https://api.test.com/.well-known/jwks.json", 3600);
        let result = verifier.verify("not.a.valid.token").await;
        assert!(result.is_err());
    }

    // Additional tests with mock JWKS server using mockito or wiremock
}
```

### Step 2: Verify

```bash
cd sdks/rust && cargo test
```

### Step 3: Commit

```bash
git commit -m "test(rust-sdk): add token verification tests"
```

---

## Task 9: React Component Rendering Tests

### Problem
6 React components (SignIn, SignUp, UserButton, UserProfile, OrgSwitcher, OrgProfile) have 0 rendering tests.

### Files
- Create: `sdks/react/tests/components.test.tsx`
- Modify: `sdks/react/package.json` — ensure @testing-library/react is in devDependencies

### Step 1: Verify test dependencies

Check that `@testing-library/react` and `@testing-library/jest-dom` are in devDependencies. If not, install:

```bash
cd sdks/react && npm install -D @testing-library/react @testing-library/jest-dom
```

### Step 2: Create component rendering tests

```tsx
// sdks/react/tests/components.test.tsx
import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { SignIn } from '../src/components/sign-in'
import { SignUp } from '../src/components/sign-up'
import { UserButton } from '../src/components/user-button'

// Mock the hooks
vi.mock('../src/hooks/use-sign-in', () => ({
  useSignIn: () => ({ signIn: vi.fn(), isLoading: false, error: null }),
}))
vi.mock('../src/hooks/use-oauth', () => ({
  useOAuth: () => ({ signInWithOAuth: vi.fn(), isLoading: false, error: null }),
}))
vi.mock('../src/hooks/use-mfa', () => ({
  useMfa: () => ({ verifyTotp: vi.fn(), isLoading: false, error: null }),
}))
vi.mock('../src/provider', () => ({
  useNucleus: () => ({
    user: null, isLoaded: true, isSignedIn: false,
    _api: {}, _sessionManager: {}, _setUser: vi.fn(),
  }),
}))
vi.mock('../src/components/appearance', () => ({
  useStyles: () => ({
    card: {}, title: {}, input: {}, button: {},
    secondaryButton: {}, error: {},
  }),
  Divider: ({ text }: { text: string }) => <hr />,
}))

describe('SignIn component', () => {
  it('renders email and password fields', () => {
    render(<SignIn />)
    expect(screen.getByPlaceholderText('Email')).toBeDefined()
    expect(screen.getByPlaceholderText('Password')).toBeDefined()
  })

  it('renders sign in button', () => {
    render(<SignIn />)
    expect(screen.getByRole('button', { name: 'Sign In' })).toBeDefined()
  })

  it('renders OAuth buttons when providers given', () => {
    render(<SignIn oauthProviders={['google', 'github']} />)
    expect(screen.getByText(/Google/i)).toBeDefined()
    expect(screen.getByText(/Github/i)).toBeDefined()
  })
})

describe('SignUp component', () => {
  it('renders email and password fields', () => {
    render(<SignUp />)
    expect(screen.getByPlaceholderText('Email')).toBeDefined()
    expect(screen.getByPlaceholderText('Password')).toBeDefined()
  })
})

describe('UserButton component', () => {
  it('returns null when not signed in', () => {
    const { container } = render(<UserButton />)
    expect(container.innerHTML).toBe('')
  })
})
```

### Step 3: Verify

```bash
cd sdks/react && npx vitest run
```

### Step 4: Commit

```bash
git commit -m "test(react): add component rendering tests for SignIn, SignUp, UserButton"
```

---

## Task 10: Next.js Component and Middleware Tests

### Problem
Next.js has 0 component tests and 0 middleware tests. `server/middleware.ts` protects routes — critical for production.

### Files
- Create: `sdks/nextjs/tests/components.test.tsx`
- Create: `sdks/nextjs/tests/middleware.test.ts`

### Step 1: Create component tests

Same pattern as React — mock hooks, render components, verify DOM output. Use `'use client'`-compatible test setup.

### Step 2: Create middleware.test.ts

```typescript
// sdks/nextjs/tests/middleware.test.ts
import { describe, it, expect, vi } from 'vitest'

// Mock next/server
vi.mock('next/server', () => ({
  NextResponse: {
    next: vi.fn(() => ({ headers: new Map() })),
    redirect: vi.fn((url: URL) => ({ url, status: 307 })),
  },
}))

describe('authMiddleware', () => {
  it('allows requests to public routes', async () => {
    // Test that /sign-in, /sign-up bypass auth
  })

  it('redirects unauthenticated requests to sign-in', async () => {
    // Test that requests without session cookie redirect
  })

  it('allows authenticated requests through', async () => {
    // Test that requests with valid session cookie proceed
  })
})
```

### Step 3: Verify

```bash
cd sdks/nextjs && npx vitest run
```

### Step 4: Commit

```bash
git commit -m "test(nextjs): add component rendering and middleware tests"
```

---

## Verification Checklist

After all tasks complete:

```bash
# Rust backend
cargo test --workspace

# Python
cd sdks/python && pytest tests/ -v

# Go
cd sdks/go && go test ./... -v -count=1

# Java
cd sdks/java && mvn test

# .NET
cd sdks/dotnet && dotnet test

# Rust SDK
cd sdks/rust && cargo test

# React
cd sdks/react && npx vitest run

# Next.js
cd sdks/nextjs && npx vitest run

# JS (ensure existing tests still pass)
cd sdks/js && npx vitest run
```

All must pass before PR.

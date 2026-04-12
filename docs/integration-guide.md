# Nucleus Integration Guide

This guide explains how to integrate Nucleus authentication into your application.

## 1. Project Setup

1. Create a project in the Nucleus dashboard
2. Note your API keys:
   - **Publishable key** (`pk_...`) — safe for client-side use
   - **Secret key** (`sk_...`) — server-side only, never expose to clients

## 2. JWT Verification

Nucleus issues short-lived RS256 JWTs (default: 5 minutes). Your backend must verify these tokens on every authenticated request.

### Fetch Public Keys

Nucleus publishes signing keys via the standard JWKS endpoint:

```
GET https://<your-nucleus-host>/.well-known/jwks.json
```

Response:

```json
{
  "keys": [
    {
      "kty": "RSA",
      "kid": "<key-id>",
      "alg": "RS256",
      "use": "sig",
      "n": "<modulus-base64url>",
      "e": "<exponent-base64url>"
    }
  ]
}
```

**Best practice:** Cache the JWKS response (it returns `Cache-Control: public, max-age=86400`) and refresh periodically or when you encounter an unknown `kid`.

### Validate Tokens

When verifying a JWT:

1. Decode the header to get the `kid`
2. Look up the matching key from your cached JWKS
3. Verify the RS256 signature
4. Validate standard claims: `exp` (not expired), `iss` (matches your Nucleus URL), `aud` (matches your project ID)

## 3. Claims Reference

The `NucleusClaims` JWT payload contains:

### Standard Claims

| Claim | Type | Description |
|-------|------|-------------|
| `sub` | `string` | User ID (UUID) |
| `iss` | `string` | Issuer URL (your Nucleus instance) |
| `aud` | `string` | Project ID (UUID) |
| `exp` | `integer` | Expiry timestamp (Unix epoch) |
| `iat` | `integer` | Issued-at timestamp (Unix epoch) |
| `jti` | `string` | Unique token ID |

### Profile Claims (optional)

| Claim | Type | Description |
|-------|------|-------------|
| `email` | `string` | User's email address |
| `first_name` | `string` | First name |
| `last_name` | `string` | Last name |
| `avatar_url` | `string` | Profile picture URL |
| `email_verified` | `boolean` | Whether the email has been verified |
| `metadata` | `object` | Arbitrary user metadata |

### Organization Claims (optional)

Present when the user is acting within an organization context:

| Claim | Type | Description |
|-------|------|-------------|
| `org_id` | `string` | Organization ID (UUID) |
| `org_slug` | `string` | Organization slug |
| `org_role` | `string` | User's role in the organization |
| `org_permissions` | `string[]` | Granted permissions within the organization |

## 4. OpenID Connect Discovery

Nucleus exposes a standard OIDC discovery document:

```
GET https://<your-nucleus-host>/.well-known/openid-configuration
```

This returns endpoints for authorization, token exchange, user info, and JWKS. Compatible OIDC client libraries can auto-configure from this URL.

Key fields:

| Field | Value |
|-------|-------|
| `issuer` | Your Nucleus instance URL |
| `authorization_endpoint` | `{issuer}/api/v1/auth/sign-in/oauth` |
| `token_endpoint` | `{issuer}/api/v1/auth/token/refresh` |
| `userinfo_endpoint` | `{issuer}/api/v1/users/me` |
| `jwks_uri` | `{issuer}/.well-known/jwks.json` |
| `id_token_signing_alg_values_supported` | `["RS256"]` |
| `scopes_supported` | `["openid", "email", "profile"]` |

## 5. Middleware Examples

### Rust (Axum)

```rust
use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    aud: String,
    exp: i64,
    email: Option<String>,
    org_id: Option<String>,
    org_role: Option<String>,
}

async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let token = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Load your public key from JWKS (cache this!)
    let decoding_key = DecodingKey::from_rsa_pem(PUBLIC_KEY_PEM)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&["<your-project-id>"]);
    validation.set_issuer(&["https://<your-nucleus-host>"]);

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    req.extensions_mut().insert(token_data.claims);
    Ok(next.run(req).await)
}
```

### Node.js (Express)

```javascript
import jwt from "jsonwebtoken";
import jwksClient from "jwks-rsa";

const client = jwksClient({
  jwksUri: "https://<your-nucleus-host>/.well-known/jwks.json",
  cache: true,
  rateLimit: true,
});

function getKey(header, callback) {
  client.getSigningKey(header.kid, (err, key) => {
    if (err) return callback(err);
    callback(null, key.getPublicKey());
  });
}

function authMiddleware(req, res, next) {
  const authHeader = req.headers.authorization;
  if (!authHeader?.startsWith("Bearer ")) {
    return res.status(401).json({ error: "Missing token" });
  }

  const token = authHeader.slice(7);

  jwt.verify(
    token,
    getKey,
    {
      algorithms: ["RS256"],
      audience: "<your-project-id>",
      issuer: "https://<your-nucleus-host>",
    },
    (err, decoded) => {
      if (err) return res.status(401).json({ error: "Invalid token" });
      req.user = decoded;
      next();
    }
  );
}
```

## 6. OAuth Redirect Setup

To use social login (Google, GitHub, etc.):

1. In the Nucleus dashboard, navigate to your project's **OAuth Providers** settings
2. Configure the provider with your OAuth client ID and secret
3. Add your application's callback URL to the **Allowed Redirect URIs** list
4. Initiate OAuth flow via:

```
POST /api/v1/auth/sign-in/oauth
Content-Type: application/json

{
  "provider": "google",
  "redirect_uri": "https://your-app.com/auth/callback"
}
```

5. Nucleus handles the OAuth exchange and redirects back to your `redirect_uri` with authentication tokens

### Redirect URI Requirements

- Must use HTTPS in production (HTTP allowed for `localhost` during development)
- Must exactly match one of the URIs registered in your project settings
- Query parameters are preserved during the redirect

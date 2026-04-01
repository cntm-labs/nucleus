# Phase 5: Missing Features — Clerk Competitive Parity

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add the features that Clerk has but Nucleus doesn't: 5 additional OAuth providers (Facebook, Discord, Twitter/X, LinkedIn, Slack), email/SMS delivery service integration (SendGrid + Twilio), i18n/localization for all React/Next.js components, and SAML Enterprise SSO foundation.

**Architecture:** OAuth providers follow the existing `OAuthProvider` trait pattern — each is a standalone module implementing 3 methods. Email/SMS uses a trait-based `NotificationService` so the delivery backend is swappable. i18n uses a lightweight context-based approach (no heavy library — just a `useTranslation` hook with a locale JSON map). SAML is implemented as a new auth handler using the `samael` crate for XML/assertion parsing.

**Tech Stack:** Rust (reqwest for OAuth, lettre/reqwest for email, samael for SAML), React (context + JSON locales), TypeScript

**Execution order:** Email/SMS delivery (unblocks existing flows) → OAuth providers (high user demand) → i18n (UX polish) → SAML (enterprise feature)

---

## Task 1: Email Delivery Service (SendGrid)

### Problem
Password reset, magic link, and OTP handlers generate tokens but never send them. Three `TODO: Send email` comments block these features from working.

### Files
- Create: `crates/nucleus-core/src/notification.rs` — trait definition
- Create: `crates/nucleus-server/src/services/email.rs` — SendGrid implementation
- Create: `crates/nucleus-server/src/services/mod.rs`
- Modify: `crates/nucleus-core/src/lib.rs` — export notification module
- Modify: `crates/nucleus-server/src/config.rs` — add email config
- Modify: `crates/nucleus-server/src/state.rs` — add NotificationService to AppState
- Modify: `crates/nucleus-server/src/main.rs` — initialize email service
- Modify: `crates/nucleus-server/Cargo.toml` — add reqwest dependency (already present)

### Step 1: Define NotificationService trait in nucleus-core

```rust
// crates/nucleus-core/src/notification.rs
use crate::error::AppError;
use async_trait::async_trait;

#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn send_email(&self, to: &str, subject: &str, html_body: &str, text_body: &str) -> Result<(), AppError>;
    async fn send_sms(&self, to: &str, body: &str) -> Result<(), AppError>;
}
```

### Step 2: Implement SendGrid email sender

```rust
// crates/nucleus-server/src/services/email.rs
use nucleus_core::notification::NotificationService;
use nucleus_core::error::AppError;
use async_trait::async_trait;

pub struct SendGridService {
    api_key: String,
    from_email: String,
    from_name: String,
    http_client: reqwest::Client,
}

impl SendGridService {
    pub fn new(api_key: String, from_email: String, from_name: String) -> Self {
        Self {
            api_key,
            from_email,
            from_name,
            http_client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NotificationService for SendGridService {
    async fn send_email(&self, to: &str, subject: &str, html_body: &str, text_body: &str) -> Result<(), AppError> {
        let payload = serde_json::json!({
            "personalizations": [{"to": [{"email": to}]}],
            "from": {"email": &self.from_email, "name": &self.from_name},
            "subject": subject,
            "content": [
                {"type": "text/plain", "value": text_body},
                {"type": "text/html", "value": html_body},
            ]
        });

        let resp = self.http_client
            .post("https://api.sendgrid.com/v3/mail/send")
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(anyhow::anyhow!("SendGrid error: {}", body)));
        }

        Ok(())
    }

    async fn send_sms(&self, _to: &str, _body: &str) -> Result<(), AppError> {
        // SMS not supported by SendGrid — use TwilioService for SMS
        Err(AppError::Internal(anyhow::anyhow!("SMS not supported by SendGrid service")))
    }
}

/// Fallback for development: logs emails instead of sending
pub struct LogNotificationService;

#[async_trait]
impl NotificationService for LogNotificationService {
    async fn send_email(&self, to: &str, subject: &str, _html_body: &str, _text_body: &str) -> Result<(), AppError> {
        tracing::info!(to = %to, subject = %subject, "Email would be sent (dev mode)");
        Ok(())
    }

    async fn send_sms(&self, to: &str, body: &str) -> Result<(), AppError> {
        tracing::info!(to = %to, body = %body, "SMS would be sent (dev mode)");
        Ok(())
    }
}
```

### Step 3: Add config for email service

In `config.rs`, add:

```rust
pub sendgrid_api_key: Option<String>,
pub from_email: String,
pub from_name: String,
```

Load from env:

```rust
sendgrid_api_key: env::var("SENDGRID_API_KEY").ok(),
from_email: env::var("FROM_EMAIL").unwrap_or_else(|_| "noreply@nucleus.dev".to_string()),
from_name: env::var("FROM_NAME").unwrap_or_else(|_| "Nucleus".to_string()),
```

### Step 4: Add NotificationService to AppState and initialize in main.rs

```rust
// In AppState:
pub notification_service: Arc<dyn NotificationService>,

// In main.rs:
let notification_service: Arc<dyn NotificationService> = match &config.sendgrid_api_key {
    Some(key) => Arc::new(SendGridService::new(key.clone(), config.from_email.clone(), config.from_name.clone())),
    None => {
        tracing::warn!("SENDGRID_API_KEY not set — using log-based email delivery");
        Arc::new(LogNotificationService)
    }
};
```

### Step 5: Wire email sending into password_reset, magic_link, and otp handlers

Replace the `tracing::debug!` TODO lines with actual email sending:

```rust
// In handle_request_reset (password_reset.rs):
let reset_url = PasswordResetService::build_url(&base_url, &generated.token);
state.notification_service.send_email(
    &body.email,
    "Reset your password",
    &format!("<p>Click <a href=\"{}\">here</a> to reset your password. This link expires in 1 hour.</p>", reset_url),
    &format!("Reset your password: {} (expires in 1 hour)", reset_url),
).await?;

// In handle_send_magic_link (magic_link.rs):
let magic_url = MagicLinkService::build_url(&base_url, &generated.token, &redirect_url, &[])?;
state.notification_service.send_email(
    &body.email,
    "Sign in to Nucleus",
    &format!("<p>Click <a href=\"{}\">here</a> to sign in. This link expires in 15 minutes.</p>", magic_url),
    &format!("Sign in: {} (expires in 15 minutes)", magic_url),
).await?;

// In handle_send_otp (otp.rs):
state.notification_service.send_email(
    &body.email_or_phone,
    "Your verification code",
    &format!("<p>Your verification code is: <strong>{}</strong>. It expires in 5 minutes.</p>", generated.code),
    &format!("Your verification code is: {}. It expires in 5 minutes.", generated.code),
).await?;
```

### Step 6: Update .env.example

```bash
# Email delivery (optional — falls back to log-based in dev)
# SENDGRID_API_KEY=SG.xxxx
# FROM_EMAIL=noreply@yourdomain.com
# FROM_NAME=YourApp
```

### Step 7: Write tests

Test `LogNotificationService` (always succeeds), test `SendGridService` payload construction (mock HTTP).

### Step 8: Verify and commit

Run: `cargo test --workspace`

```bash
git commit -m "feat(email): add NotificationService trait with SendGrid and log-based implementations"
```

---

## Task 2: SMS Delivery Service (Twilio)

### Problem
OTP handler supports `email_or_phone` but has no SMS delivery path.

### Files
- Create: `crates/nucleus-server/src/services/sms.rs` — Twilio implementation
- Modify: `crates/nucleus-server/src/services/mod.rs` — export
- Modify: `crates/nucleus-server/src/config.rs` — add Twilio config

### Step 1: Implement TwilioService

```rust
// crates/nucleus-server/src/services/sms.rs
pub struct TwilioService {
    account_sid: String,
    auth_token: String,
    from_number: String,
    http_client: reqwest::Client,
}

impl TwilioService {
    pub fn new(account_sid: String, auth_token: String, from_number: String) -> Self {
        Self { account_sid, auth_token, from_number, http_client: reqwest::Client::new() }
    }

    pub async fn send_sms(&self, to: &str, body: &str) -> Result<(), AppError> {
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.account_sid
        );
        let resp = self.http_client
            .post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&[("To", to), ("From", &self.from_number), ("Body", body)])
            .send()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(anyhow::anyhow!("Twilio error: {}", body)));
        }
        Ok(())
    }
}
```

### Step 2: Create CompositeNotificationService

Combines email (SendGrid) + SMS (Twilio) into one service:

```rust
pub struct CompositeNotificationService {
    email: Arc<dyn NotificationService>,
    sms: Option<Arc<TwilioService>>,
}

#[async_trait]
impl NotificationService for CompositeNotificationService {
    async fn send_email(&self, to: &str, subject: &str, html: &str, text: &str) -> Result<(), AppError> {
        self.email.send_email(to, subject, html, text).await
    }

    async fn send_sms(&self, to: &str, body: &str) -> Result<(), AppError> {
        match &self.sms {
            Some(sms) => sms.send_sms(to, body).await,
            None => {
                tracing::warn!(to = %to, "SMS not configured — message not sent");
                Ok(())
            }
        }
    }
}
```

### Step 3: Add config and wire in main.rs

```rust
pub twilio_account_sid: Option<String>,
pub twilio_auth_token: Option<String>,
pub twilio_from_number: Option<String>,
```

### Step 4: Wire SMS in OTP handler

```rust
// Detect if email or phone
if body.email_or_phone.contains('@') {
    state.notification_service.send_email(&body.email_or_phone, "Your code", &html, &text).await?;
} else {
    state.notification_service.send_sms(&body.email_or_phone, &format!("Your Nucleus code: {}", generated.code)).await?;
}
```

### Step 5: Verify and commit

```bash
git commit -m "feat(sms): add Twilio SMS service for OTP delivery"
```

---

## Task 3: Add Facebook OAuth Provider

### Files
- Create: `crates/nucleus-auth/src/oauth/facebook.rs`
- Modify: `crates/nucleus-auth/src/oauth/mod.rs` — add `pub mod facebook;`
- Test: inline `#[cfg(test)]` module

### Step 1: Implement Facebook provider

```rust
// crates/nucleus-auth/src/oauth/facebook.rs
use super::provider::*;
use crate::error::AppError;
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

pub struct FacebookProvider {
    config: OAuthConfig,
    http_client: Arc<dyn HttpClient>,
}

impl FacebookProvider {
    pub fn new(config: OAuthConfig, http_client: Arc<dyn HttpClient>) -> Self {
        Self { config, http_client }
    }

    pub fn default_scopes() -> Vec<String> {
        vec!["email".to_string(), "public_profile".to_string()]
    }
}

#[derive(Deserialize)]
struct FacebookTokenResponse { access_token: String }

#[derive(Deserialize)]
struct FacebookUserInfo {
    id: String,
    email: Option<String>,
    name: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    picture: Option<FacebookPicture>,
}

#[derive(Deserialize)]
struct FacebookPicture { data: Option<FacebookPictureData> }

#[derive(Deserialize)]
struct FacebookPictureData { url: Option<String> }

#[async_trait]
impl OAuthProvider for FacebookProvider {
    fn provider_name(&self) -> &str { "facebook" }

    fn authorization_url(&self, state: &str, pkce_challenge: Option<&str>) -> Result<AuthorizationUrl, AppError> {
        let mut url = format!(
            "https://www.facebook.com/v19.0/dialog/oauth?client_id={}&redirect_uri={}&state={}&scope={}&response_type=code",
            self.config.client_id,
            urlencoding::encode(&self.config.redirect_uri),
            state,
            self.config.scopes.join(","),
        );
        if let Some(challenge) = pkce_challenge {
            url.push_str(&format!("&code_challenge={}&code_challenge_method=S256", challenge));
        }
        Ok(AuthorizationUrl { url, pkce_verifier: None })
    }

    async fn exchange_code(&self, code: &str, pkce_verifier: Option<&str>) -> Result<OAuthUserInfo, AppError> {
        let mut params = vec![
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
            ("redirect_uri", self.config.redirect_uri.as_str()),
            ("code", code),
        ];
        if let Some(verifier) = pkce_verifier {
            params.push(("code_verifier", verifier));
        }

        let token_resp: FacebookTokenResponse = serde_json::from_str(
            &self.http_client.post_form("https://graph.facebook.com/v19.0/oauth/access_token", &params).await?,
        ).map_err(|e| AppError::Internal(e.into()))?;

        let user_resp: FacebookUserInfo = serde_json::from_str(
            &self.http_client.get_with_bearer(
                "https://graph.facebook.com/v19.0/me?fields=id,email,name,first_name,last_name,picture.type(large)",
                &token_resp.access_token,
            ).await?,
        ).map_err(|e| AppError::Internal(e.into()))?;

        Ok(OAuthUserInfo {
            provider: "facebook".to_string(),
            provider_user_id: user_resp.id,
            email: user_resp.email,
            name: user_resp.name,
            first_name: user_resp.first_name,
            last_name: user_resp.last_name,
            avatar_url: user_resp.picture.and_then(|p| p.data).and_then(|d| d.url),
            raw_data: serde_json::json!({}),
        })
    }
}
```

### Step 2: Add module export and tests

Add `pub mod facebook;` to `mod.rs`. Add unit test for `authorization_url` generation.

### Step 3: Verify and commit

```bash
git commit -m "feat(oauth): add Facebook OAuth provider"
```

---

## Task 4: Add Discord OAuth Provider

### Files
- Create: `crates/nucleus-auth/src/oauth/discord.rs`
- Modify: `crates/nucleus-auth/src/oauth/mod.rs`

### Step 1: Implement Discord provider

Same pattern as Facebook. Key differences:
- Auth URL: `https://discord.com/oauth2/authorize`
- Token URL: `https://discord.com/api/oauth2/token`
- UserInfo URL: `https://discord.com/api/users/@me`
- Scopes: `identify email`
- Avatar URL: `https://cdn.discordapp.com/avatars/{id}/{avatar}.png`
- Discord returns `username` and `global_name`

### Step 2: Verify and commit

```bash
git commit -m "feat(oauth): add Discord OAuth provider"
```

---

## Task 5: Add Twitter/X OAuth 2.0 Provider

### Files
- Create: `crates/nucleus-auth/src/oauth/twitter.rs`
- Modify: `crates/nucleus-auth/src/oauth/mod.rs`

### Step 1: Implement Twitter provider

Key differences:
- Auth URL: `https://twitter.com/i/oauth2/authorize`
- Token URL: `https://api.twitter.com/2/oauth2/token`
- UserInfo URL: `https://api.twitter.com/2/users/me?user.fields=profile_image_url,name`
- Scopes: `users.read tweet.read offline.access`
- PKCE is **required** by Twitter OAuth 2.0
- Token exchange uses Basic auth (`client_id:client_secret`)
- Response wraps user in `data` field: `{"data": {"id": "...", "name": "..."}}`

### Step 2: Verify and commit

```bash
git commit -m "feat(oauth): add Twitter/X OAuth 2.0 provider"
```

---

## Task 6: Add LinkedIn OAuth Provider

### Files
- Create: `crates/nucleus-auth/src/oauth/linkedin.rs`
- Modify: `crates/nucleus-auth/src/oauth/mod.rs`

### Step 1: Implement LinkedIn provider

Key differences:
- Auth URL: `https://www.linkedin.com/oauth/v2/authorization`
- Token URL: `https://www.linkedin.com/oauth/v2/accessToken`
- UserInfo URL: `https://api.linkedin.com/v2/userinfo` (OpenID Connect)
- Scopes: `openid profile email`
- Maps: `sub` → provider_user_id, `given_name` → first_name, `family_name` → last_name, `picture` → avatar_url

### Step 2: Verify and commit

```bash
git commit -m "feat(oauth): add LinkedIn OAuth provider"
```

---

## Task 7: Add Slack OAuth Provider

### Files
- Create: `crates/nucleus-auth/src/oauth/slack.rs`
- Modify: `crates/nucleus-auth/src/oauth/mod.rs`

### Step 1: Implement Slack provider

Key differences:
- Auth URL: `https://slack.com/openid/connect/authorize`
- Token URL: `https://slack.com/api/openid.connect.token`
- UserInfo URL: `https://slack.com/api/openid.connect.userInfo`
- Scopes: `openid profile email`
- Maps: `sub` → provider_user_id, `given_name`, `family_name`, `picture`

### Step 2: Verify and commit

```bash
git commit -m "feat(oauth): add Slack OAuth provider"
```

---

## Task 8: React SDK i18n — Localization System

### Problem
All 7 React components have 40+ hardcoded English strings. No i18n support exists.

### Files
- Create: `sdks/react/src/i18n/index.ts` — translation hook and context
- Create: `sdks/react/src/i18n/locales/en.ts` — English strings
- Create: `sdks/react/src/i18n/locales/th.ts` — Thai strings (example)
- Modify: `sdks/react/src/provider.tsx` — add locale to NucleusProvider
- Modify: `sdks/react/src/components/sign-in.tsx` — use `useTranslation()`
- Modify: `sdks/react/src/components/sign-up.tsx` — use `useTranslation()`
- Modify: `sdks/react/src/components/user-button.tsx` — use `useTranslation()`
- Modify: `sdks/react/src/components/user-profile.tsx` — use `useTranslation()`
- Modify: `sdks/react/src/components/org-switcher.tsx` — use `useTranslation()`
- Modify: `sdks/react/src/components/org-profile.tsx` — use `useTranslation()`
- Test: `sdks/react/tests/i18n.test.tsx`

### Step 1: Create lightweight i18n system

```typescript
// sdks/react/src/i18n/index.ts
import { createContext, useContext } from 'react'
import { en } from './locales/en'

export type Locale = Record<string, string>

const I18nContext = createContext<Locale>(en)

export function I18nProvider({ locale, children }: { locale?: Locale; children: React.ReactNode }) {
  return <I18nContext.Provider value={locale ?? en}>{children}</I18nContext.Provider>
}

export function useTranslation() {
  const locale = useContext(I18nContext)
  return (key: string, vars?: Record<string, string>) => {
    let text = locale[key] ?? key
    if (vars) {
      for (const [k, v] of Object.entries(vars)) {
        text = text.replace(`{{${k}}}`, v)
      }
    }
    return text
  }
}
```

### Step 2: Create English locale

```typescript
// sdks/react/src/i18n/locales/en.ts
export const en: Record<string, string> = {
  'signIn.title': 'Sign In',
  'signIn.button': 'Sign In',
  'signIn.loading': 'Signing in...',
  'signIn.oauth': 'Continue with {{provider}}',
  'signIn.email': 'Email',
  'signIn.password': 'Password',
  'signIn.mfa.prompt': 'Enter the verification code from your authenticator app',
  'signIn.mfa.button': 'Verify',
  'signIn.mfa.loading': 'Verifying...',
  'signUp.title': 'Create Account',
  'signUp.button': 'Sign Up',
  'signUp.loading': 'Creating account...',
  'signUp.firstName': 'First Name',
  'signUp.lastName': 'Last Name',
  'signUp.email': 'Email',
  'signUp.password': 'Password',
  'signUp.oauth': 'Continue with {{provider}}',
  'userButton.sessions': 'Sessions',
  'userButton.signOut': 'Sign Out',
  'userProfile.title': 'Profile',
  'userProfile.profileTab': 'Profile',
  'userProfile.passwordTab': 'Password',
  'userProfile.email': 'Email',
  'userProfile.firstName': 'First Name',
  'userProfile.lastName': 'Last Name',
  'userProfile.save': 'Save Changes',
  'userProfile.saving': 'Saving...',
  'userProfile.currentPassword': 'Current Password',
  'userProfile.newPassword': 'New Password',
  'userProfile.updatePassword': 'Update Password',
  'userProfile.updating': 'Updating...',
  'orgSwitcher.select': 'Select Organization',
  'orgSwitcher.loading': 'Loading...',
  'orgSwitcher.empty': 'No organizations',
  'orgSwitcher.create': '+ Create Organization',
  'orgSwitcher.name': 'Name',
  'orgSwitcher.slug': 'Slug',
  'orgSwitcher.cancel': 'Cancel',
  'orgSwitcher.createButton': 'Create',
  'orgProfile.noOrg': 'No organization selected',
  'orgProfile.members': 'Members',
  'orgProfile.remove': 'Remove',
  'orgProfile.invite': 'Invite Member',
  'orgProfile.email': 'Email',
  'orgProfile.roleMember': 'Member',
  'orgProfile.roleAdmin': 'Admin',
  'orgProfile.sendInvite': 'Send Invitation',
  'orgProfile.sending': 'Inviting...',
}
```

### Step 3: Create Thai locale example

```typescript
// sdks/react/src/i18n/locales/th.ts
export const th: Record<string, string> = {
  'signIn.title': 'เข้าสู่ระบบ',
  'signIn.button': 'เข้าสู่ระบบ',
  'signIn.loading': 'กำลังเข้าสู่ระบบ...',
  'signIn.oauth': 'ดำเนินการต่อด้วย {{provider}}',
  'signIn.email': 'อีเมล',
  'signIn.password': 'รหัสผ่าน',
  'signUp.title': 'สร้างบัญชี',
  'signUp.button': 'สมัครสมาชิก',
  // ... remaining translations
}
```

### Step 4: Update NucleusProvider to accept locale

```tsx
// In provider.tsx, wrap children with I18nProvider:
<I18nContext.Provider value={props.locale ?? en}>
  {children}
</I18nContext.Provider>
```

API for users:

```tsx
import { th } from '@cntm-labs/react/i18n/locales/th'

<NucleusProvider publishableKey="pk_..." locale={th}>
  <SignIn />
</NucleusProvider>
```

### Step 5: Update all components to use `useTranslation()`

Replace hardcoded strings in every component. Example for sign-in.tsx:

```tsx
// OLD:
<h2 style={s.title}>Sign In</h2>
<button>{isLoading ? 'Signing in...' : 'Sign In'}</button>

// NEW:
const t = useTranslation()
<h2 style={s.title}>{t('signIn.title')}</h2>
<button>{isLoading ? t('signIn.loading') : t('signIn.button')}</button>
```

### Step 6: Copy i18n system to Next.js SDK

Copy `i18n/` directory to `sdks/nextjs/src/i18n/` and update Next.js components the same way.

### Step 7: Write i18n tests

```typescript
// sdks/react/tests/i18n.test.tsx
import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { I18nProvider, useTranslation } from '../src/i18n'
import { th } from '../src/i18n/locales/th'

function TestComponent() {
  const t = useTranslation()
  return <span>{t('signIn.title')}</span>
}

describe('i18n', () => {
  it('defaults to English', () => {
    render(<I18nProvider><TestComponent /></I18nProvider>)
    expect(screen.getByText('Sign In')).toBeDefined()
  })

  it('uses provided locale', () => {
    render(<I18nProvider locale={th}><TestComponent /></I18nProvider>)
    expect(screen.getByText('เข้าสู่ระบบ')).toBeDefined()
  })

  it('interpolates variables', () => {
    function VarComponent() {
      const t = useTranslation()
      return <span>{t('signIn.oauth', { provider: 'Google' })}</span>
    }
    render(<I18nProvider><VarComponent /></I18nProvider>)
    expect(screen.getByText('Continue with Google')).toBeDefined()
  })
})
```

### Step 8: Verify and commit

```bash
cd sdks/react && npx vitest run
cd ../nextjs && npx vitest run
```

```bash
git commit -m "feat(react,nextjs): add i18n localization system with English and Thai locales"
```

---

## Task 9: SAML Enterprise SSO Foundation

### Problem
Clerk supports SAML SSO for enterprise customers. Nucleus has no SAML support at all.

### Files
- Create: `crates/nucleus-auth/src/saml/mod.rs`
- Create: `crates/nucleus-auth/src/saml/service.rs`
- Create: `crates/nucleus-auth/src/saml/handler.rs`
- Modify: `crates/nucleus-auth/src/lib.rs` — add `pub mod saml`
- Modify: `crates/nucleus-auth/Cargo.toml` — add `samael` dependency
- Modify: `crates/nucleus-server/src/router.rs` — add SAML routes
- Create: `crates/nucleus-migrate/migrations/015_create_saml_connections.sql`

### Step 1: Add migration for SAML connections

```sql
-- crates/nucleus-migrate/migrations/015_create_saml_connections.sql
CREATE TABLE saml_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id),
    name VARCHAR(255) NOT NULL,
    entity_id VARCHAR(512) NOT NULL,
    sso_url TEXT NOT NULL,
    certificate TEXT NOT NULL,
    attribute_mapping JSONB NOT NULL DEFAULT '{"email": "email", "first_name": "firstName", "last_name": "lastName"}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(project_id, entity_id)
);

CREATE INDEX idx_saml_connections_project ON saml_connections(project_id);
```

### Step 2: Add samael dependency

In `crates/nucleus-auth/Cargo.toml`:

```toml
samael = { version = "0.0.17", features = ["xmlsec"] }
```

### Step 3: Implement SAML service

```rust
// crates/nucleus-auth/src/saml/service.rs
use samael::metadata::EntityDescriptor;
use samael::service_provider::ServiceProvider;

pub struct SamlConfig {
    pub entity_id: String,
    pub sso_url: String,
    pub certificate: String,
    pub attribute_mapping: AttributeMapping,
}

pub struct AttributeMapping {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

pub struct SamlService;

impl SamlService {
    /// Parse SAML Response XML and extract user attributes
    pub fn parse_response(
        saml_response: &str,
        certificate: &str,
    ) -> Result<SamlUserInfo, AppError> {
        // Decode base64 SAML response
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(saml_response)
            .map_err(|e| AppError::Validation(format!("Invalid SAML response: {}", e)))?;

        let xml = String::from_utf8(decoded)
            .map_err(|e| AppError::Validation(format!("Invalid UTF-8 in SAML response: {}", e)))?;

        // Parse and verify signature using samael
        let response = samael::schema::Response::from_xml(&xml)
            .map_err(|e| AppError::Auth(AuthError::SamlInvalid(format!("Failed to parse: {}", e))))?;

        // Extract assertions and attributes
        let assertion = response.assertions.first()
            .ok_or(AppError::Auth(AuthError::SamlInvalid("No assertion found".into())))?;

        let attrs = &assertion.attribute_statements;
        // Map attributes to SamlUserInfo based on attribute_mapping config
        // ...

        Ok(SamlUserInfo {
            name_id: assertion.subject.name_id.value.clone(),
            email: extract_attr(attrs, "email"),
            first_name: extract_attr(attrs, "firstName"),
            last_name: extract_attr(attrs, "lastName"),
        })
    }

    /// Generate SP metadata XML for IdP configuration
    pub fn generate_metadata(
        entity_id: &str,
        acs_url: &str,
    ) -> String {
        format!(r#"<?xml version="1.0"?>
<EntityDescriptor xmlns="urn:oasis:names:tc:SAML:2.0:metadata" entityID="{}">
  <SPSSODescriptor AuthnRequestsSigned="false" WantAssertionsSigned="true"
    protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
    <AssertionConsumerService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
      Location="{}" index="0" isDefault="true"/>
  </SPSSODescriptor>
</EntityDescriptor>"#, entity_id, acs_url)
    }
}
```

### Step 4: Implement SAML handlers

```rust
// crates/nucleus-auth/src/saml/handler.rs

/// POST /api/v1/auth/saml/callback — Assertion Consumer Service (ACS)
pub async fn handle_saml_callback(
    State(state): State<SamlState>,
    Form(body): Form<SamlCallbackRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user_info = SamlService::parse_response(&body.saml_response, &saml_config.certificate)?;

    // Find or create user by email (same pattern as OAuth)
    let user = find_or_create_user(&state, &user_info).await?;

    // Create session + JWT
    let session = state.session_service.create_session(&user.id, &project_id, None, None, None).await?;
    let jwt = state.auth_service.issue_jwt(&user, &session)?;

    // Redirect to app with token
    Ok(Redirect::to(&format!("{}?token={}", redirect_url, jwt.token)))
}

/// GET /api/v1/auth/saml/metadata — SP metadata for IdP configuration
pub async fn handle_saml_metadata(
    State(state): State<SamlState>,
) -> impl IntoResponse {
    let metadata = SamlService::generate_metadata(&entity_id, &acs_url);
    ([(header::CONTENT_TYPE, "application/xml")], metadata)
}
```

### Step 5: Add routes to router.rs

```rust
// In auth routes:
.route("/auth/saml/callback", post(saml::handle_saml_callback))
.route("/auth/saml/metadata", get(saml::handle_saml_metadata))
```

### Step 6: Write tests

Test SAML response parsing, attribute extraction, metadata generation.

### Step 7: Verify and commit

```bash
cargo test --workspace
git commit -m "feat(auth): add SAML Enterprise SSO foundation with assertion parsing and SP metadata"
```

---

## Verification Checklist

After all tasks complete:

```bash
# Full Rust test suite
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings

# React SDK tests
cd sdks/react && npx vitest run

# Next.js SDK tests
cd sdks/nextjs && npx vitest run

# Verify all 9 OAuth providers exist
ls crates/nucleus-auth/src/oauth/*.rs | wc -l
# Expected: 10 (mod.rs + provider.rs + 4 existing + 5 new = 11, minus mod.rs/provider.rs = 9 providers)

# Verify no TODO email sending remains
grep -rn "TODO.*Send.*email\|TODO.*email.*service\|TODO.*wire" crates/nucleus-auth/src/handlers/ --include="*.rs"
# Expected: no results

# Verify i18n strings
grep -rn "useTranslation" sdks/react/src/components/ | wc -l
# Expected: 6+ (one per component file)
```

All must pass before PR.

# Incomplete SDKs Implementation — Full Clerk Parity Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement 5 incomplete SDKs (JS browser, React, Next.js, Flutter, Android Java) to full Clerk parity with all authentication, session, organization, and UI features.

**Context:** Nucleus is a Clerk alternative. All SDKs must provide the same level of functionality as Clerk's SDKs for their respective platforms.

## Layer Architecture

```
  Layer 1 (Core)     — HTTP client, token storage, session, auth flows (ALL SDKs)
  Layer 2 (Backend)  — JWT verify, middleware, route protection (server SDKs only)
  Layer 3 (UI)       — Pre-built components with appearance theming (client SDKs only)
```

| SDK | Layer 1 | Layer 2 | Layer 3 |
|-----|---------|---------|---------|
| @cntm-labs/js | HTTP client, localStorage, multi-session, OAuth popup, WebAuthn, MFA, profile, org RBAC | — | — |
| @cntm-labs/react | Auth state, hooks, OAuth, passkeys, MFA, multi-session, profile, verification, org RBAC | — | SignIn, SignUp, UserProfile, UserButton, OrgSwitcher, OrgProfile, theming |
| @cntm-labs/nextjs | Same as React | Edge middleware, getAuth(), currentUser(), route protection, JWT verify | Same as React + server components |
| nucleus_flutter | HTTP client, secure storage, OAuth (url_launcher), MFA, multi-session, profile, org RBAC | — | SignIn, SignUp, UserButton, UserProfile, OrgSwitcher, OrgProfile widgets, theming |
| nucleus-android-java | OkHttp client, EncryptedSharedPreferences, OAuth (Custom Tabs), MFA, multi-session, profile, org RBAC | — | XML views: SignInView, SignUpView, ProfileView, UserButtonView, OrgSwitcherView |

## Feature Matrix (All 5 SDKs must implement)

### Core Features (Layer 1)
- Email/password sign-in/sign-up
- OAuth social login (Google, GitHub, etc.)
- Passkeys (WebAuthn / platform-specific)
- MFA — TOTP setup/verify, SMS verify, backup codes
- Email/phone verification flow
- User profile update (name, avatar, email, password)
- Token storage (platform-appropriate)
- Session restore on startup
- Auto token refresh (60s before expiry)
- Multi-session (multiple accounts, switch between)
- Active sessions list + revoke
- Organization list, switch, create
- Organization invitations (invite/accept/revoke)
- Organization roles & permissions (RBAC)

### Backend Features (Layer 2 — Next.js only among the 5)
- JWT verification via JWKS (jose library)
- Edge middleware route protection
- `getAuth()` / `currentUser()` server helpers
- `verifyNucleusToken()` implementation

### UI Features (Layer 3 — React, Next.js, Flutter, Android Java)
- `SignIn` — email/password + OAuth buttons + passkey + MFA step
- `SignUp` — registration + email verification step
- `UserProfile` — edit name/avatar/email/password, MFA setup, active sessions
- `UserButton` — avatar dropdown with profile/sign-out/session switch
- `OrgSwitcher` — org list, create org, switch active
- `OrgProfile` — members list, invite, roles, permissions
- Appearance/theme customization — color scheme, border radius, font, custom CSS

## API Endpoints

### Auth
```
POST /v1/auth/sign-in              { email, password }
POST /v1/auth/sign-up              { email, password, first_name, last_name }
POST /v1/auth/sign-out
POST /v1/auth/passkey/challenge
POST /v1/auth/passkey/verify
GET  /v1/oauth/{provider}/authorize
POST /v1/oauth/token               { code, redirect_uri }
```

### MFA
```
POST /v1/auth/mfa/totp/setup       -> { secret, qr_uri }
POST /v1/auth/mfa/totp/verify      { code }
POST /v1/auth/mfa/sms/send         { phone }
POST /v1/auth/mfa/sms/verify       { code }
GET  /v1/auth/mfa/backup-codes     -> { codes[] }
```

### Verification
```
POST /v1/auth/verify-email/send    { email }
POST /v1/auth/verify-email/confirm { code }
POST /v1/auth/verify-phone/send    { phone }
POST /v1/auth/verify-phone/confirm { code }
```

### Sessions
```
GET    /v1/sessions                -> { sessions[] }
GET    /v1/sessions/{id}
POST   /v1/sessions/refresh        { refresh_token }
DELETE /v1/sessions/{id}           (revoke)
POST   /v1/sessions/switch         { session_id }
```

### User Profile
```
GET   /v1/user
PATCH /v1/user                     { first_name, last_name, avatar_url }
PUT   /v1/user/password            { current_password, new_password }
PUT   /v1/user/email               { email }
```

### Organizations
```
GET    /v1/organizations
POST   /v1/organizations           { name, slug }
GET    /v1/organizations/{id}
GET    /v1/organizations/{id}/members
POST   /v1/organizations/{id}/invitations   { email, role }
PUT    /v1/organizations/{id}/invitations/{id}  { accept/revoke }
PUT    /v1/organizations/{id}/members/{id}/role  { role }
DELETE /v1/organizations/{id}/members/{id}
```

### Request Headers
- Client SDKs: `X-Nucleus-Publishable-Key: pk_...`
- Authenticated requests: `Authorization: Bearer <accessToken>`
- Server SDKs (admin): `Authorization: Bearer <secretKey>`

## Appearance Theming (UI SDKs)

All UI SDKs support a consistent theming API:

```typescript
// React/Next.js pattern
<NucleusProvider
  publishableKey="pk_..."
  appearance={{
    theme: 'light' | 'dark' | 'auto',
    variables: {
      colorPrimary: '#6366f1',
      borderRadius: '0.5rem',
      fontFamily: 'Inter, sans-serif',
    },
    elements: {
      signInCard: { backgroundColor: '#fff' },
      primaryButton: { fontSize: '14px' },
    }
  }}
>
```

Flutter: `NucleusTheme` class with equivalent properties.
Android Java: XML attributes + `NucleusAppearance` builder.

## Per-SDK Gap Analysis

### @cntm-labs/js (STUB -> Full Core)
- Delete Web Component stubs (headless-only)
- Implement `NucleusApi` HTTP client with all endpoints
- Implement `TokenStorage` (localStorage)
- Implement `SessionManager` with multi-session + auto-refresh
- Implement OAuth popup flow + passkey WebAuthn flow
- Implement MFA, verification, profile, org RBAC
- Expand types to full Clerk parity

### @cntm-labs/react (PARTIAL -> Full)
- Fix: session restore (real getSession), token refresh, org list API call
- Add: OAuth (useSignIn.signInWithOAuth), passkeys (useSignIn.signInWithPasskey)
- Add: MFA hooks (useMFA), verification hooks, profile hooks
- Add: multi-session (useSessionList, switchSession)
- Add: org invitations, roles, permissions hooks
- Fix: OrgSwitcher component (wire to API)
- Implement: UserProfile, OrgProfile components
- Implement: appearance theming system

### @cntm-labs/nextjs (PARTIAL -> Full)
- All React fixes/additions above
- Implement: currentUser(), auth() with real JWT verification
- Implement: authMiddleware() with actual token verification + redirect
- Implement: verifyNucleusToken() using jose JWKS
- Wire: session utilities (already exist, unused)
- Fix: useSignIn/useSignUp (currently TODO stubs)

### nucleus_flutter (STUB -> Full)
- Add: Nucleus singleton entry point
- Implement: signIn/signUp/signOut in NucleusAuth using NucleusApiClient
- Wire: TokenStorage + AutoRefresh to lifecycle
- Add: OAuth (url_launcher + deep link), MFA, verification, profile, org RBAC
- Implement: all 6 widgets (SignIn, SignUp, UserButton, UserProfile, OrgSwitcher, OrgProfile)
- Implement: NucleusTheme appearance system

### nucleus-android-java (STUB -> Full)
- Add: OkHttp ApiClient (mirror Kotlin SDK)
- Implement: NucleusAuth with real HTTP calls + callbacks
- Add: NucleusSession model, token storage (EncryptedSharedPreferences)
- Add: session restore, auto-refresh, OAuth (Custom Tabs), MFA, profile, org RBAC
- Implement: XML views (SignInView, SignUpView, ProfileView, UserButtonView, OrgSwitcherView)
- Add: NucleusAppearance theming builder

## Reference SDKs
- Node.js SDK: server-side patterns, types, API client
- Android Kotlin SDK: mobile patterns, OAuth, passkeys, UI components, session management
- React SDK (working parts): hooks pattern, provider pattern

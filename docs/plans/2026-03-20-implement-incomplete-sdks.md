# Implement 5 Incomplete SDKs — Full Clerk Parity

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement JS browser, React, Next.js, Flutter, and Android Java SDKs to full Clerk parity — auth flows, MFA, multi-session, org RBAC, UI components, and appearance theming.

**Architecture:** Each SDK is self-contained (no cross-SDK dependencies). All share the same API endpoints and request patterns. JS SDK is headless-only (Layer 1). React/Next.js/Flutter/Android Java include UI components (Layer 3). Next.js adds server-side middleware (Layer 2). Reference the functional Node.js and Android Kotlin SDKs for patterns.

**Tech Stack:** TypeScript (JS/React/Next.js), Dart (Flutter), Java + Android SDK (Android Java), jose (JWT), WebAuthn (passkeys), OAuth popup/redirect

**Execution order:** JS → React → Next.js → Flutter → Android Java (each builds on patterns established by previous)

---

## SDK 1: @cntm-labs/js (Headless TypeScript Client)

### Task 1.1: Core types and API client

**Files:**
- Rewrite: `sdks/js/src/types.ts`
- Create: `sdks/js/src/api.ts`
- Keep: `sdks/js/src/verify.ts`

**Step 1: Rewrite types.ts with full Clerk-parity types**

```typescript
// sdks/js/src/types.ts
export interface NucleusConfig {
  publishableKey: string
  baseUrl?: string
}

export interface NucleusUser {
  id: string
  email: string
  email_verified: boolean
  phone?: string
  phone_verified: boolean
  first_name?: string
  last_name?: string
  avatar_url?: string
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface NucleusSession {
  id: string
  token: string
  refresh_token: string
  expires_at: string
  user_id: string
}

export interface NucleusOrganization {
  id: string
  name: string
  slug: string
  created_at: string
}

export interface NucleusMember {
  id: string
  user_id: string
  org_id: string
  role: string
  permissions: string[]
  email: string
  first_name?: string
  last_name?: string
}

export interface NucleusInvitation {
  id: string
  org_id: string
  email: string
  role: string
  status: 'pending' | 'accepted' | 'revoked'
  created_at: string
}

export interface NucleusClaims {
  sub: string
  iss: string
  aud: string
  exp: number
  iat: number
  jti: string
  email: string
  first_name?: string
  last_name?: string
  avatar_url?: string
  email_verified: boolean
  metadata: Record<string, unknown>
  org_id?: string
  org_slug?: string
  org_role?: string
  org_permissions?: string[]
}

export interface NucleusMfaSetup {
  secret: string
  qr_uri: string
}

export interface NucleusAuthResponse {
  user: NucleusUser
  session: NucleusSession
}

export type OAuthProvider = 'google' | 'github' | 'apple' | 'microsoft' | 'discord' | 'slack'
```

**Step 2: Create api.ts — full HTTP client covering all endpoints**

Mirror the React SDK's `NucleusApi` pattern but extend with ALL Clerk-parity endpoints:

```typescript
// sdks/js/src/api.ts
import type {
  NucleusUser, NucleusSession, NucleusOrganization,
  NucleusMember, NucleusInvitation, NucleusMfaSetup,
  NucleusAuthResponse, OAuthProvider,
} from './types'

const DEFAULT_BASE_URL = 'https://api.nucleus.dev'

export class NucleusApi {
  private publishableKey: string
  private baseUrl: string

  constructor(publishableKey: string, baseUrl?: string) {
    this.publishableKey = publishableKey
    this.baseUrl = (baseUrl ?? DEFAULT_BASE_URL).replace(/\/$/, '')
  }

  private async request<T>(path: string, init: RequestInit = {}): Promise<T> {
    const resp = await fetch(`${this.baseUrl}${path}`, {
      ...init,
      headers: {
        'Content-Type': 'application/json',
        'X-Nucleus-Publishable-Key': this.publishableKey,
        ...init.headers,
      },
    })
    if (!resp.ok) {
      const body = await resp.text()
      throw new NucleusApiError(resp.status, body)
    }
    if (resp.status === 204) return undefined as T
    return resp.json() as Promise<T>
  }

  private authed(token: string): Record<string, string> {
    return { Authorization: `Bearer ${token}` }
  }

  // --- Auth ---
  signIn(email: string, password: string) {
    return this.request<NucleusAuthResponse>('/v1/auth/sign-in', {
      method: 'POST', body: JSON.stringify({ email, password }),
    })
  }

  signUp(email: string, password: string, firstName?: string, lastName?: string) {
    return this.request<NucleusAuthResponse>('/v1/auth/sign-up', {
      method: 'POST',
      body: JSON.stringify({ email, password, first_name: firstName, last_name: lastName }),
    })
  }

  signOut(token: string) {
    return this.request<void>('/v1/auth/sign-out', {
      method: 'POST', headers: this.authed(token),
    })
  }

  // --- OAuth ---
  getOAuthUrl(provider: OAuthProvider, redirectUri: string) {
    const params = new URLSearchParams({ redirect_uri: redirectUri })
    return `${this.baseUrl}/v1/oauth/${provider}/authorize?${params}&publishable_key=${this.publishableKey}`
  }

  exchangeOAuthCode(code: string, redirectUri: string) {
    return this.request<NucleusAuthResponse>('/v1/oauth/token', {
      method: 'POST', body: JSON.stringify({ code, redirect_uri: redirectUri }),
    })
  }

  // --- Passkeys ---
  getPasskeyChallenge(token: string) {
    return this.request<{ challenge: string; rp: object; user: object }>('/v1/auth/passkey/challenge', {
      method: 'POST', headers: this.authed(token),
    })
  }

  verifyPasskey(credential: object) {
    return this.request<NucleusAuthResponse>('/v1/auth/passkey/verify', {
      method: 'POST', body: JSON.stringify(credential),
    })
  }

  // --- MFA ---
  mfaTotpSetup(token: string) {
    return this.request<NucleusMfaSetup>('/v1/auth/mfa/totp/setup', {
      method: 'POST', headers: this.authed(token),
    })
  }

  mfaTotpVerify(token: string, code: string) {
    return this.request<{ verified: boolean }>('/v1/auth/mfa/totp/verify', {
      method: 'POST', headers: this.authed(token), body: JSON.stringify({ code }),
    })
  }

  mfaSmsSend(token: string, phone: string) {
    return this.request<void>('/v1/auth/mfa/sms/send', {
      method: 'POST', headers: this.authed(token), body: JSON.stringify({ phone }),
    })
  }

  mfaSmsVerify(token: string, code: string) {
    return this.request<{ verified: boolean }>('/v1/auth/mfa/sms/verify', {
      method: 'POST', headers: this.authed(token), body: JSON.stringify({ code }),
    })
  }

  mfaBackupCodes(token: string) {
    return this.request<{ codes: string[] }>('/v1/auth/mfa/backup-codes', {
      headers: this.authed(token),
    })
  }

  // --- Verification ---
  sendEmailVerification(token: string, email: string) {
    return this.request<void>('/v1/auth/verify-email/send', {
      method: 'POST', headers: this.authed(token), body: JSON.stringify({ email }),
    })
  }

  confirmEmailVerification(token: string, code: string) {
    return this.request<{ verified: boolean }>('/v1/auth/verify-email/confirm', {
      method: 'POST', headers: this.authed(token), body: JSON.stringify({ code }),
    })
  }

  sendPhoneVerification(token: string, phone: string) {
    return this.request<void>('/v1/auth/verify-phone/send', {
      method: 'POST', headers: this.authed(token), body: JSON.stringify({ phone }),
    })
  }

  confirmPhoneVerification(token: string, code: string) {
    return this.request<{ verified: boolean }>('/v1/auth/verify-phone/confirm', {
      method: 'POST', headers: this.authed(token), body: JSON.stringify({ code }),
    })
  }

  // --- Sessions ---
  getSessions(token: string) {
    return this.request<NucleusSession[]>('/v1/sessions', { headers: this.authed(token) })
  }

  getSession(token: string, sessionId: string) {
    return this.request<NucleusSession>(`/v1/sessions/${sessionId}`, { headers: this.authed(token) })
  }

  refreshSession(refreshToken: string) {
    return this.request<NucleusSession>('/v1/sessions/refresh', {
      method: 'POST', body: JSON.stringify({ refresh_token: refreshToken }),
    })
  }

  revokeSession(token: string, sessionId: string) {
    return this.request<void>(`/v1/sessions/${sessionId}`, {
      method: 'DELETE', headers: this.authed(token),
    })
  }

  switchSession(token: string, sessionId: string) {
    return this.request<NucleusSession>('/v1/sessions/switch', {
      method: 'POST', headers: this.authed(token), body: JSON.stringify({ session_id: sessionId }),
    })
  }

  // --- User Profile ---
  getUser(token: string) {
    return this.request<NucleusUser>('/v1/user', { headers: this.authed(token) })
  }

  updateUser(token: string, data: { first_name?: string; last_name?: string; avatar_url?: string }) {
    return this.request<NucleusUser>('/v1/user', {
      method: 'PATCH', headers: this.authed(token), body: JSON.stringify(data),
    })
  }

  updatePassword(token: string, currentPassword: string, newPassword: string) {
    return this.request<void>('/v1/user/password', {
      method: 'PUT', headers: this.authed(token),
      body: JSON.stringify({ current_password: currentPassword, new_password: newPassword }),
    })
  }

  updateEmail(token: string, email: string) {
    return this.request<void>('/v1/user/email', {
      method: 'PUT', headers: this.authed(token), body: JSON.stringify({ email }),
    })
  }

  // --- Organizations ---
  getOrganizations(token: string) {
    return this.request<NucleusOrganization[]>('/v1/organizations', { headers: this.authed(token) })
  }

  createOrganization(token: string, name: string, slug: string) {
    return this.request<NucleusOrganization>('/v1/organizations', {
      method: 'POST', headers: this.authed(token), body: JSON.stringify({ name, slug }),
    })
  }

  getOrganization(token: string, orgId: string) {
    return this.request<NucleusOrganization>(`/v1/organizations/${orgId}`, { headers: this.authed(token) })
  }

  getMembers(token: string, orgId: string) {
    return this.request<NucleusMember[]>(`/v1/organizations/${orgId}/members`, { headers: this.authed(token) })
  }

  removeMember(token: string, orgId: string, memberId: string) {
    return this.request<void>(`/v1/organizations/${orgId}/members/${memberId}`, {
      method: 'DELETE', headers: this.authed(token),
    })
  }

  updateMemberRole(token: string, orgId: string, memberId: string, role: string) {
    return this.request<NucleusMember>(`/v1/organizations/${orgId}/members/${memberId}/role`, {
      method: 'PUT', headers: this.authed(token), body: JSON.stringify({ role }),
    })
  }

  createInvitation(token: string, orgId: string, email: string, role: string) {
    return this.request<NucleusInvitation>(`/v1/organizations/${orgId}/invitations`, {
      method: 'POST', headers: this.authed(token), body: JSON.stringify({ email, role }),
    })
  }

  updateInvitation(token: string, orgId: string, invitationId: string, action: 'accept' | 'revoke') {
    return this.request<NucleusInvitation>(`/v1/organizations/${orgId}/invitations/${invitationId}`, {
      method: 'PUT', headers: this.authed(token), body: JSON.stringify({ action }),
    })
  }
}

export class NucleusApiError extends Error {
  constructor(public status: number, public body: string) {
    super(`Nucleus API error (${status}): ${body}`)
    this.name = 'NucleusApiError'
  }
}
```

**Step 3: Verify TypeScript compiles**

Run: `cd sdks/js && npx tsc --noEmit`

**Step 4: Commit**

```bash
git add sdks/js/src/types.ts sdks/js/src/api.ts
git commit -m "feat(js): add full API client and types for Clerk parity"
```

---

### Task 1.2: Token storage and session manager

**Files:**
- Create: `sdks/js/src/storage.ts`
- Create: `sdks/js/src/session.ts`

**Step 1: Create storage.ts — localStorage-backed multi-session token storage**

Pattern: Store active session + list of all sessions. Keys prefixed with `__nucleus_`.

```typescript
// sdks/js/src/storage.ts
const PREFIX = '__nucleus_'

export const TokenStorage = {
  getActiveToken(): string | null {
    return localStorage.getItem(`${PREFIX}token`)
  },
  getRefreshToken(): string | null {
    return localStorage.getItem(`${PREFIX}refresh_token`)
  },
  getExpiresAt(): string | null {
    return localStorage.getItem(`${PREFIX}expires_at`)
  },
  getActiveSessionId(): string | null {
    return localStorage.getItem(`${PREFIX}session_id`)
  },
  setSession(session: { id: string; token: string; refresh_token: string; expires_at: string }): void {
    localStorage.setItem(`${PREFIX}token`, session.token)
    localStorage.setItem(`${PREFIX}refresh_token`, session.refresh_token)
    localStorage.setItem(`${PREFIX}expires_at`, session.expires_at)
    localStorage.setItem(`${PREFIX}session_id`, session.id)
    // Multi-session: add to session list
    const sessions = TokenStorage.getSessionIds()
    if (!sessions.includes(session.id)) {
      sessions.push(session.id)
      localStorage.setItem(`${PREFIX}sessions`, JSON.stringify(sessions))
    }
  },
  clear(): void {
    const id = TokenStorage.getActiveSessionId()
    localStorage.removeItem(`${PREFIX}token`)
    localStorage.removeItem(`${PREFIX}refresh_token`)
    localStorage.removeItem(`${PREFIX}expires_at`)
    localStorage.removeItem(`${PREFIX}session_id`)
    // Remove from multi-session list
    if (id) {
      const sessions = TokenStorage.getSessionIds().filter(s => s !== id)
      localStorage.setItem(`${PREFIX}sessions`, JSON.stringify(sessions))
    }
  },
  clearAll(): void {
    Object.keys(localStorage)
      .filter(k => k.startsWith(PREFIX))
      .forEach(k => localStorage.removeItem(k))
  },
  getSessionIds(): string[] {
    try {
      return JSON.parse(localStorage.getItem(`${PREFIX}sessions`) ?? '[]')
    } catch {
      return []
    }
  },
}
```

**Step 2: Create session.ts — auto-refresh + session lifecycle**

```typescript
// sdks/js/src/session.ts
import type { NucleusSession } from './types'
import { TokenStorage } from './storage'
import type { NucleusApi } from './api'

const REFRESH_BUFFER_MS = 60_000 // refresh 60s before expiry

export class SessionManager {
  private refreshTimer: ReturnType<typeof setTimeout> | null = null
  private api: NucleusApi
  private onChange: (session: NucleusSession | null) => void

  constructor(api: NucleusApi, onChange: (session: NucleusSession | null) => void) {
    this.api = api
    this.onChange = onChange
  }

  async restore(): Promise<NucleusSession | null> {
    const token = TokenStorage.getActiveToken()
    const refreshToken = TokenStorage.getRefreshToken()
    const expiresAt = TokenStorage.getExpiresAt()
    const sessionId = TokenStorage.getActiveSessionId()
    if (!token || !refreshToken || !expiresAt || !sessionId) return null

    if (this.isExpired(expiresAt)) {
      return this.refresh(refreshToken)
    }

    const session: NucleusSession = { id: sessionId, token, refresh_token: refreshToken, expires_at: expiresAt, user_id: '' }
    this.scheduleRefresh(session)
    return session
  }

  setSession(session: NucleusSession): void {
    TokenStorage.setSession(session)
    this.scheduleRefresh(session)
    this.onChange(session)
  }

  async refresh(refreshToken?: string): Promise<NucleusSession | null> {
    const rt = refreshToken ?? TokenStorage.getRefreshToken()
    if (!rt) { this.clearSession(); return null }
    try {
      const session = await this.api.refreshSession(rt)
      TokenStorage.setSession(session)
      this.scheduleRefresh(session)
      this.onChange(session)
      return session
    } catch {
      this.clearSession()
      return null
    }
  }

  clearSession(): void {
    this.cancelRefresh()
    TokenStorage.clear()
    this.onChange(null)
  }

  private scheduleRefresh(session: NucleusSession): void {
    this.cancelRefresh()
    const expiresMs = new Date(session.expires_at).getTime()
    const refreshAt = expiresMs - REFRESH_BUFFER_MS - Date.now()
    if (refreshAt <= 0) {
      this.refresh(session.refresh_token)
      return
    }
    this.refreshTimer = setTimeout(() => this.refresh(session.refresh_token), refreshAt)
  }

  private cancelRefresh(): void {
    if (this.refreshTimer) { clearTimeout(this.refreshTimer); this.refreshTimer = null }
  }

  private isExpired(expiresAt: string): boolean {
    return new Date(expiresAt).getTime() <= Date.now()
  }

  destroy(): void {
    this.cancelRefresh()
  }
}
```

**Step 3: Commit**

```bash
git add sdks/js/src/storage.ts sdks/js/src/session.ts
git commit -m "feat(js): add token storage and session manager with auto-refresh"
```

---

### Task 1.3: Nucleus client — main entry point with all auth flows

**Files:**
- Rewrite: `sdks/js/src/nucleus.ts`
- Create: `sdks/js/src/oauth.ts`
- Create: `sdks/js/src/passkey.ts`

**Step 1: Create oauth.ts — popup-based OAuth flow**

```typescript
// sdks/js/src/oauth.ts
import type { NucleusAuthResponse, OAuthProvider } from './types'
import type { NucleusApi } from './api'

export function startOAuthFlow(
  api: NucleusApi,
  provider: OAuthProvider,
): Promise<NucleusAuthResponse> {
  return new Promise((resolve, reject) => {
    const redirectUri = `${window.location.origin}/__nucleus/oauth/callback`
    const url = api.getOAuthUrl(provider, redirectUri)
    const popup = window.open(url, 'nucleus-oauth', 'width=500,height=700,popup=true')
    if (!popup) { reject(new Error('Failed to open OAuth popup')); return }

    const onMessage = async (event: MessageEvent) => {
      if (event.origin !== window.location.origin) return
      if (event.data?.type !== 'nucleus:oauth:callback') return
      window.removeEventListener('message', onMessage)
      const { code, error } = event.data
      if (error) { reject(new Error(error)); return }
      try {
        const result = await api.exchangeOAuthCode(code, redirectUri)
        resolve(result)
      } catch (e) { reject(e) }
    }
    window.addEventListener('message', onMessage)

    // Poll for popup close
    const poll = setInterval(() => {
      if (popup.closed) { clearInterval(poll); window.removeEventListener('message', onMessage); reject(new Error('OAuth popup closed')) }
    }, 500)
  })
}
```

**Step 2: Create passkey.ts — WebAuthn passkey flow**

```typescript
// sdks/js/src/passkey.ts
import type { NucleusAuthResponse } from './types'
import type { NucleusApi } from './api'

export async function signInWithPasskey(api: NucleusApi, token: string): Promise<NucleusAuthResponse> {
  const challenge = await api.getPasskeyChallenge(token)
  const credential = await navigator.credentials.get({
    publicKey: {
      challenge: Uint8Array.from(atob(challenge.challenge as unknown as string), c => c.charCodeAt(0)),
      rpId: window.location.hostname,
      userVerification: 'preferred',
    },
  }) as PublicKeyCredential
  const response = credential.response as AuthenticatorAssertionResponse
  return api.verifyPasskey({
    id: credential.id,
    rawId: btoa(String.fromCharCode(...new Uint8Array(credential.rawId))),
    response: {
      authenticatorData: btoa(String.fromCharCode(...new Uint8Array(response.authenticatorData))),
      clientDataJSON: btoa(String.fromCharCode(...new Uint8Array(response.clientDataJSON))),
      signature: btoa(String.fromCharCode(...new Uint8Array(response.signature))),
    },
    type: credential.type,
  })
}
```

**Step 3: Rewrite nucleus.ts — full Nucleus client**

```typescript
// sdks/js/src/nucleus.ts
import type { NucleusConfig, NucleusUser, NucleusSession, NucleusOrganization, NucleusMember, NucleusInvitation, NucleusMfaSetup, OAuthProvider } from './types'
import { NucleusApi } from './api'
import { SessionManager } from './session'
import { TokenStorage } from './storage'
import { startOAuthFlow } from './oauth'
import { signInWithPasskey } from './passkey'

export class Nucleus {
  private static api: NucleusApi
  private static sessionManager: SessionManager
  private static _user: NucleusUser | null = null
  private static _session: NucleusSession | null = null
  private static _organization: NucleusOrganization | null = null
  private static _listeners: Set<() => void> = new Set()
  private static _configured = false

  static async configure(config: NucleusConfig): Promise<void> {
    Nucleus.api = new NucleusApi(config.publishableKey, config.baseUrl)
    Nucleus.sessionManager = new SessionManager(Nucleus.api, (session) => {
      Nucleus._session = session
      Nucleus._notify()
    })
    Nucleus._configured = true

    // Restore session
    const session = await Nucleus.sessionManager.restore()
    if (session) {
      Nucleus._session = session
      try { Nucleus._user = await Nucleus.api.getUser(session.token) } catch { /* ignore */ }
      Nucleus._notify()
    }
  }

  // --- State ---
  static get user() { return Nucleus._user }
  static get session() { return Nucleus._session }
  static get organization() { return Nucleus._organization }
  static get isSignedIn() { return Nucleus._user !== null && Nucleus._session !== null }

  static getToken(): string | null { return Nucleus._session?.token ?? null }

  static addListener(fn: () => void): () => void {
    Nucleus._listeners.add(fn)
    return () => Nucleus._listeners.delete(fn)
  }

  private static _notify(): void {
    Nucleus._listeners.forEach(fn => fn())
  }

  // --- Auth ---
  static async signIn(email: string, password: string) {
    const { user, session } = await Nucleus.api.signIn(email, password)
    Nucleus._user = user; Nucleus.sessionManager.setSession(session)
    return { user, session }
  }

  static async signUp(email: string, password: string, firstName?: string, lastName?: string) {
    const { user, session } = await Nucleus.api.signUp(email, password, firstName, lastName)
    Nucleus._user = user; Nucleus.sessionManager.setSession(session)
    return { user, session }
  }

  static async signOut() {
    const token = Nucleus.getToken()
    if (token) { try { await Nucleus.api.signOut(token) } catch { /* best effort */ } }
    Nucleus._user = null; Nucleus._organization = null; Nucleus.sessionManager.clearSession()
  }

  static async signInWithOAuth(provider: OAuthProvider) {
    const { user, session } = await startOAuthFlow(Nucleus.api, provider)
    Nucleus._user = user; Nucleus.sessionManager.setSession(session)
    return { user, session }
  }

  static async signInWithPasskey() {
    const token = Nucleus.getToken()
    if (!token) throw new Error('Must be signed in to register passkey')
    const { user, session } = await signInWithPasskey(Nucleus.api, token)
    Nucleus._user = user; Nucleus.sessionManager.setSession(session)
    return { user, session }
  }

  // --- MFA ---
  static async mfaSetupTotp() {
    const token = Nucleus.getToken()!
    return Nucleus.api.mfaTotpSetup(token)
  }

  static async mfaVerifyTotp(code: string) {
    const token = Nucleus.getToken()!
    return Nucleus.api.mfaTotpVerify(token, code)
  }

  static async mfaSendSms(phone: string) {
    const token = Nucleus.getToken()!
    return Nucleus.api.mfaSmsSend(token, phone)
  }

  static async mfaVerifySms(code: string) {
    const token = Nucleus.getToken()!
    return Nucleus.api.mfaSmsVerify(token, code)
  }

  static async mfaGetBackupCodes() {
    const token = Nucleus.getToken()!
    return Nucleus.api.mfaBackupCodes(token)
  }

  // --- Verification ---
  static async sendEmailVerification(email: string) {
    return Nucleus.api.sendEmailVerification(Nucleus.getToken()!, email)
  }

  static async confirmEmailVerification(code: string) {
    return Nucleus.api.confirmEmailVerification(Nucleus.getToken()!, code)
  }

  static async sendPhoneVerification(phone: string) {
    return Nucleus.api.sendPhoneVerification(Nucleus.getToken()!, phone)
  }

  static async confirmPhoneVerification(code: string) {
    return Nucleus.api.confirmPhoneVerification(Nucleus.getToken()!, code)
  }

  // --- User Profile ---
  static async updateProfile(data: { first_name?: string; last_name?: string; avatar_url?: string }) {
    Nucleus._user = await Nucleus.api.updateUser(Nucleus.getToken()!, data)
    Nucleus._notify()
    return Nucleus._user
  }

  static async updatePassword(currentPassword: string, newPassword: string) {
    return Nucleus.api.updatePassword(Nucleus.getToken()!, currentPassword, newPassword)
  }

  static async updateEmail(email: string) {
    return Nucleus.api.updateEmail(Nucleus.getToken()!, email)
  }

  // --- Sessions (multi-session) ---
  static async getSessions() {
    return Nucleus.api.getSessions(Nucleus.getToken()!)
  }

  static async revokeSession(sessionId: string) {
    return Nucleus.api.revokeSession(Nucleus.getToken()!, sessionId)
  }

  static async switchSession(sessionId: string) {
    const session = await Nucleus.api.switchSession(Nucleus.getToken()!, sessionId)
    Nucleus.sessionManager.setSession(session)
    Nucleus._user = await Nucleus.api.getUser(session.token)
    Nucleus._notify()
    return session
  }

  static getSessionIds(): string[] {
    return TokenStorage.getSessionIds()
  }

  // --- Organizations ---
  static async getOrganizations() {
    return Nucleus.api.getOrganizations(Nucleus.getToken()!)
  }

  static async createOrganization(name: string, slug: string) {
    return Nucleus.api.createOrganization(Nucleus.getToken()!, name, slug)
  }

  static async setActiveOrganization(org: NucleusOrganization | null) {
    Nucleus._organization = org
    Nucleus._notify()
  }

  static async getMembers(orgId: string) {
    return Nucleus.api.getMembers(Nucleus.getToken()!, orgId)
  }

  static async inviteMember(orgId: string, email: string, role: string) {
    return Nucleus.api.createInvitation(Nucleus.getToken()!, orgId, email, role)
  }

  static async updateInvitation(orgId: string, invitationId: string, action: 'accept' | 'revoke') {
    return Nucleus.api.updateInvitation(Nucleus.getToken()!, orgId, invitationId, action)
  }

  static async updateMemberRole(orgId: string, memberId: string, role: string) {
    return Nucleus.api.updateMemberRole(Nucleus.getToken()!, orgId, memberId, role)
  }

  static async removeMember(orgId: string, memberId: string) {
    return Nucleus.api.removeMember(Nucleus.getToken()!, orgId, memberId)
  }
}
```

**Step 4: Update index.ts — clean exports, remove Web Component stubs**

```typescript
// sdks/js/src/index.ts
const VERSION = '0.1.0-dev.1';
if (VERSION.includes('-dev')) {
  console.warn(`[Nucleus] WARNING: You are using a dev preview (${VERSION}). Do not use in production.`);
}

export { Nucleus } from './nucleus'
export { NucleusApi, NucleusApiError } from './api'
export { verifyToken } from './verify'
export { TokenStorage } from './storage'
export type {
  NucleusConfig, NucleusUser, NucleusSession, NucleusOrganization,
  NucleusMember, NucleusInvitation, NucleusClaims, NucleusMfaSetup,
  NucleusAuthResponse, OAuthProvider,
} from './types'
```

**Step 5: Delete old Web Component stubs**

Delete: `sdks/js/src/components/sign-in.ts`, `sdks/js/src/components/sign-up.ts`, `sdks/js/src/components/user-button.ts`

**Step 6: Add tsconfig.json if missing, verify build**

Run: `cd sdks/js && npx tsc --noEmit`
Run: `cd sdks/js && npm run build`

**Step 7: Commit**

```bash
git add sdks/js/src/ && git rm sdks/js/src/components/*.ts
git commit -m "feat(js): implement full headless Nucleus client with Clerk parity

- Email/password, OAuth popup, WebAuthn passkeys
- MFA (TOTP, SMS, backup codes)
- Email/phone verification
- User profile management
- Multi-session support
- Organization RBAC (members, invitations, roles)"
```

---

## SDK 2: @cntm-labs/react (Fix + Full Features + UI Components)

### Task 2.1: Fix core — session restore, token refresh, org list

**Files:**
- Modify: `sdks/react/src/provider.tsx`
- Modify: `sdks/react/src/client/api.ts` — extend with all endpoints (same pattern as JS SDK's api.ts)
- Modify: `sdks/react/src/client/types.ts` — expand types (copy from JS SDK's types.ts)
- Create: `sdks/react/src/client/session.ts` — session management (adapted from Next.js SDK's session.ts)

**Steps:** Expand `NucleusApi` with all endpoints from Task 1.1. Expand types. Fix `NucleusProvider` to use real `getSession()` call. Add auto-refresh timer. Fix org loading.

**Commit:** `fix(react): fix session restore, add token refresh and org loading`

### Task 2.2: Add OAuth, passkey, MFA hooks

**Files:**
- Create: `sdks/react/src/hooks/use-oauth.ts`
- Create: `sdks/react/src/hooks/use-passkey.ts`
- Create: `sdks/react/src/hooks/use-mfa.ts`
- Create: `sdks/react/src/hooks/use-verification.ts`
- Create: `sdks/react/src/hooks/use-profile.ts`
- Create: `sdks/react/src/hooks/use-session-list.ts`
- Create: `sdks/react/src/hooks/use-organization-list.ts`

**Steps:** Each hook follows existing pattern (useState + async actions + error handling). Reference `use-sign-in.ts` as the template.

**Commit:** `feat(react): add OAuth, passkey, MFA, verification, profile, and session hooks`

### Task 2.3: Implement UI components + appearance theming

**Files:**
- Create: `sdks/react/src/components/appearance.tsx` — theme context + CSS variable injection
- Modify: `sdks/react/src/components/sign-in.tsx` — add OAuth buttons, passkey button, MFA step
- Modify: `sdks/react/src/components/sign-up.tsx` — add email verification step
- Create: `sdks/react/src/components/user-profile.tsx`
- Create: `sdks/react/src/components/org-profile.tsx`
- Modify: `sdks/react/src/components/org-switcher.tsx` — wire to API
- Modify: `sdks/react/src/components/user-button.tsx` — add multi-session switch

**Steps:** Each component uses hooks from Task 2.2. Appearance theming via CSS custom properties injected by `<NucleusProvider appearance={...}>`.

**Commit:** `feat(react): implement UI components with appearance theming`

### Task 2.4: Update exports + verify build

**Commit:** `chore(react): update exports and verify build`

---

## SDK 3: @cntm-labs/nextjs (Fix + Server-Side + Full Features)

### Task 3.1: Fix client-side — wire useSignIn/useSignUp, session from cookies

**Files:**
- Modify: `sdks/nextjs/src/hooks/use-sign-in.ts` — implement real API calls
- Modify: `sdks/nextjs/src/hooks/use-sign-up.ts` — implement real API calls
- Modify: `sdks/nextjs/src/hooks/use-organization-list.ts` — implement API call
- Modify: `sdks/nextjs/src/provider.tsx` — restore session from cookies on mount

**Commit:** `fix(nextjs): wire useSignIn/useSignUp hooks and session restore`

### Task 3.2: Implement server-side auth

**Files:**
- Rewrite: `sdks/nextjs/src/server/token.ts` — real JWT verification with jose
- Rewrite: `sdks/nextjs/src/server/auth.ts` — real `currentUser()` and `auth()` reading cookies
- Rewrite: `sdks/nextjs/src/server/middleware.ts` — real token verification + redirect

**Step 1: Implement verifyNucleusToken**

```typescript
// sdks/nextjs/src/server/token.ts
import * as jose from 'jose'
import type { NucleusClaims } from '../client/types'

let jwks: ReturnType<typeof jose.createRemoteJWKSet> | null = null

export async function verifyNucleusToken(token: string, baseUrl: string): Promise<NucleusClaims> {
  if (!jwks) {
    jwks = jose.createRemoteJWKSet(new URL(`${baseUrl}/.well-known/jwks.json`))
  }
  const { payload } = await jose.jwtVerify(token, jwks, { algorithms: ['RS256'] })
  return payload as unknown as NucleusClaims
}
```

**Step 2: Implement currentUser() and auth()**

Read `__nucleus_session` cookie, verify JWT, return claims.

**Step 3: Implement authMiddleware()**

Check token on protected routes, redirect to sign-in if missing/invalid.

**Commit:** `feat(nextjs): implement server-side JWT verification and auth middleware`

### Task 3.3: Add all hooks + UI components (same as React SDK pattern)

Port OAuth, passkey, MFA, verification, profile, session list hooks from React SDK (same pattern). Wire UI components. Add appearance theming.

**Commit:** `feat(nextjs): add full hook set and UI components`

---

## SDK 4: nucleus_flutter (Dart — Full Implementation)

### Task 4.1: Create Nucleus singleton + implement auth flows in NucleusAuth

**Files:**
- Create: `sdks/flutter/lib/src/nucleus.dart` — singleton entry point
- Modify: `sdks/flutter/lib/src/auth/auth_state.dart` — add signIn, signUp, signOut, OAuth, MFA methods
- Modify: `sdks/flutter/lib/src/client.dart` — add all API endpoints
- Wire: `TokenStorage` and `AutoRefresh` to lifecycle

**Steps:** Follow Android Kotlin SDK's `Nucleus.kt` pattern. `NucleusAuth` becomes the state holder + API caller. `Nucleus.configure()` initializes everything + restores session.

**Commit:** `feat(flutter): implement Nucleus singleton and full auth flows`

### Task 4.2: Implement OAuth (url_launcher) + deep link callback

**Files:**
- Create: `sdks/flutter/lib/src/auth/oauth.dart`

**Steps:** Use `url_launcher` to open browser for OAuth. Handle deep link callback via platform channel or `uni_links` package.

**Commit:** `feat(flutter): add OAuth flow with url_launcher`

### Task 4.3: Implement MFA, verification, profile, org RBAC

**Files:**
- Create: `sdks/flutter/lib/src/auth/mfa.dart`
- Create: `sdks/flutter/lib/src/auth/verification.dart`
- Add methods to `NucleusAuth` for profile update, org management

**Commit:** `feat(flutter): add MFA, verification, profile, and org RBAC`

### Task 4.4: Implement all widgets

**Files:**
- Rewrite: `sdks/flutter/lib/src/widgets/sign_in_widget.dart`
- Rewrite: `sdks/flutter/lib/src/widgets/sign_up_widget.dart`
- Rewrite: `sdks/flutter/lib/src/widgets/user_button.dart`
- Create: `sdks/flutter/lib/src/widgets/user_profile.dart`
- Create: `sdks/flutter/lib/src/widgets/org_switcher.dart`
- Create: `sdks/flutter/lib/src/widgets/org_profile.dart`
- Create: `sdks/flutter/lib/src/theme/nucleus_theme.dart`

**Steps:** Each widget is a `StatefulWidget` using `NucleusProvider.of(context)` to access auth state. Follow Material Design patterns. `NucleusTheme` provides appearance customization.

**Commit:** `feat(flutter): implement all UI widgets with theming`

### Task 4.5: Update exports + verify

**Commit:** `chore(flutter): update exports and verify build`

---

## SDK 5: nucleus-android-java (Java — Full Implementation)

### Task 5.1: Add OkHttp ApiClient + models

**Files:**
- Create: `sdks/android-java/src/main/java/dev/nucleus/network/ApiClient.java`
- Create: `sdks/android-java/src/main/java/dev/nucleus/model/NucleusSession.java`
- Create: `sdks/android-java/src/main/java/dev/nucleus/model/NucleusOrganization.java`
- Create: `sdks/android-java/src/main/java/dev/nucleus/model/NucleusMember.java`
- Create: `sdks/android-java/src/main/java/dev/nucleus/model/NucleusInvitation.java`

**Steps:** Mirror Kotlin SDK's `ApiClient.kt`. Use OkHttp + org.json for JSON parsing (avoid Gson/Moshi for minimal deps). Add OkHttp dependency to `build.gradle`.

**Commit:** `feat(android-java): add OkHttp API client and data models`

### Task 5.2: Implement NucleusAuth + session management

**Files:**
- Rewrite: `sdks/android-java/src/main/java/dev/nucleus/NucleusAuth.java`
- Create: `sdks/android-java/src/main/java/dev/nucleus/session/TokenStorage.java`
- Create: `sdks/android-java/src/main/java/dev/nucleus/session/SessionManager.java`

**Steps:** `NucleusAuth` methods call `ApiClient`, store tokens via `TokenStorage` (EncryptedSharedPreferences), schedule refresh via `SessionManager`. All async methods use `NucleusCallback<T>` pattern.

**Commit:** `feat(android-java): implement NucleusAuth with session management`

### Task 5.3: Add OAuth (Custom Tabs) + MFA + org RBAC

**Files:**
- Create: `sdks/android-java/src/main/java/dev/nucleus/auth/OAuthManager.java`
- Create: `sdks/android-java/src/main/java/dev/nucleus/auth/MfaManager.java`

**Steps:** OAuth via Chrome Custom Tabs (same as Kotlin SDK). MFA methods on NucleusAuth. Org management methods.

**Commit:** `feat(android-java): add OAuth, MFA, and org RBAC`

### Task 5.4: Implement XML views

**Files:**
- Rewrite: `sdks/android-java/src/main/java/dev/nucleus/ui/NucleusSignInView.java`
- Create: `sdks/android-java/src/main/java/dev/nucleus/ui/NucleusSignUpView.java`
- Create: `sdks/android-java/src/main/java/dev/nucleus/ui/NucleusProfileView.java`
- Create: `sdks/android-java/src/main/java/dev/nucleus/ui/NucleusUserButtonView.java`
- Create: `sdks/android-java/src/main/java/dev/nucleus/ui/NucleusOrgSwitcherView.java`
- Create: `sdks/android-java/src/main/java/dev/nucleus/ui/NucleusAppearance.java`

**Steps:** Each view extends appropriate ViewGroup (LinearLayout/FrameLayout). Programmatic layout creation (no XML layout files needed). `NucleusAppearance` builder for theming.

**Commit:** `feat(android-java): implement XML views with appearance theming`

### Task 5.5: Update build.gradle + verify

Add OkHttp + EncryptedSharedPreferences dependencies. Verify build.

**Commit:** `chore(android-java): update dependencies and verify build`

---

## SDK 6: Fix CI workflow build issues (from dry-run failures)

### Task 6.1: Fix npm build (add tsconfig/tsup config where missing)

Ensure `sdks/js`, `sdks/react`, `sdks/nextjs` all have working `tsup.config.ts` and `tsconfig.json`.

### Task 6.2: Fix Python build

Ensure `sdks/python` has correct `pyproject.toml` build config for `hatchling`.

### Task 6.3: Fix Rust SDK standalone build

Add workspace exclusion or standalone build config.

### Task 6.4: Fix Flutter pub publish

Ensure `pubspec.yaml` dependencies resolve.

### Task 6.5: Fix .NET pack (README path)

Fix `PackageReadmeFile` path in `.csproj` files.

### Task 6.6: Fix Go module (go.sum)

Run `go mod tidy` to generate proper `go.sum`.

### Task 6.7: Re-run dry-run publish and verify all green

```bash
gh workflow run sdk-publish.yml -f sdk=all -f version=0.1.0-dev.1 -f dry_run=true
```

**Commit:** `fix(ci): fix all SDK build issues for publish workflow`

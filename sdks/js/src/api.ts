import type {
  NucleusUser, NucleusSession, NucleusOrganization,
  NucleusMember, NucleusInvitation, NucleusMfaSetup,
  NucleusAuthResponse, OAuthProvider,
} from './types'
import { validateSignIn, validateSignUp } from './validation'

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
    validateSignIn(email, password)
    return this.request<NucleusAuthResponse>('/v1/auth/sign-in', {
      method: 'POST', body: JSON.stringify({ email, password }),
    })
  }

  signUp(email: string, password: string, firstName?: string, lastName?: string) {
    validateSignUp(email, password)
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
  getOAuthUrl(provider: OAuthProvider, redirectUri: string, state?: string) {
    const params = new URLSearchParams({ redirect_uri: redirectUri })
    if (state) params.set('state', state)
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

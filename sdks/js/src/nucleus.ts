import type { NucleusConfig, NucleusUser, NucleusSession, NucleusOrganization, OAuthProvider } from './types'
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

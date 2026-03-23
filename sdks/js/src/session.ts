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

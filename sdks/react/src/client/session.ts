import type { NucleusSession } from './types'
import type { NucleusApi } from './api'

const PREFIX = '__nucleus_'
const REFRESH_BUFFER_MS = 60_000

export class SessionManager {
  private refreshTimer: ReturnType<typeof setTimeout> | null = null
  private api: NucleusApi
  private onChange: (session: NucleusSession | null) => void

  constructor(api: NucleusApi, onChange: (session: NucleusSession | null) => void) {
    this.api = api
    this.onChange = onChange
  }

  restore(): { token: string; refresh_token: string; expires_at: string; session_id: string } | null {
    if (typeof window === 'undefined') return null
    const token = localStorage.getItem(`${PREFIX}token`)
    const refresh_token = localStorage.getItem(`${PREFIX}refresh_token`)
    const expires_at = localStorage.getItem(`${PREFIX}expires_at`)
    const session_id = localStorage.getItem(`${PREFIX}session_id`)
    if (!token || !refresh_token || !expires_at || !session_id) return null
    return { token, refresh_token, expires_at, session_id }
  }

  setSession(session: NucleusSession): void {
    if (typeof window !== 'undefined') {
      localStorage.setItem(`${PREFIX}token`, session.token)
      localStorage.setItem(`${PREFIX}refresh_token`, session.refresh_token)
      localStorage.setItem(`${PREFIX}expires_at`, session.expires_at)
      localStorage.setItem(`${PREFIX}session_id`, session.id)
      const ids = this.getSessionIds()
      if (!ids.includes(session.id)) {
        ids.push(session.id)
        localStorage.setItem(`${PREFIX}sessions`, JSON.stringify(ids))
      }
    }
    this.scheduleRefresh(session)
    this.onChange(session)
  }

  async refresh(refreshToken?: string): Promise<NucleusSession | null> {
    const rt = refreshToken ?? (typeof window !== 'undefined' ? localStorage.getItem(`${PREFIX}refresh_token`) : null)
    if (!rt) { this.clearSession(); return null }
    try {
      const session = await this.api.refreshSession(rt)
      this.setSession(session)
      return session
    } catch {
      this.clearSession()
      return null
    }
  }

  clearSession(): void {
    this.cancelRefresh()
    if (typeof window !== 'undefined') {
      const id = localStorage.getItem(`${PREFIX}session_id`)
      localStorage.removeItem(`${PREFIX}token`)
      localStorage.removeItem(`${PREFIX}refresh_token`)
      localStorage.removeItem(`${PREFIX}expires_at`)
      localStorage.removeItem(`${PREFIX}session_id`)
      if (id) {
        const ids = this.getSessionIds().filter(s => s !== id)
        localStorage.setItem(`${PREFIX}sessions`, JSON.stringify(ids))
      }
    }
    this.onChange(null)
  }

  getSessionIds(): string[] {
    if (typeof window === 'undefined') return []
    try {
      return JSON.parse(localStorage.getItem(`${PREFIX}sessions`) ?? '[]')
    } catch { return [] }
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

  destroy(): void {
    this.cancelRefresh()
  }
}

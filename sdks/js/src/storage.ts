const PREFIX = '__nucleus_'

function hasStorage(): boolean {
  return typeof localStorage !== 'undefined'
}

export const TokenStorage = {
  getActiveToken(): string | null {
    if (!hasStorage()) return null
    return localStorage.getItem(`${PREFIX}token`)
  },
  getRefreshToken(): string | null {
    if (!hasStorage()) return null
    return localStorage.getItem(`${PREFIX}refresh_token`)
  },
  getExpiresAt(): string | null {
    if (!hasStorage()) return null
    return localStorage.getItem(`${PREFIX}expires_at`)
  },
  getActiveSessionId(): string | null {
    if (!hasStorage()) return null
    return localStorage.getItem(`${PREFIX}session_id`)
  },
  setSession(session: { id: string; token: string; refresh_token: string; expires_at: string }): void {
    if (!hasStorage()) return
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
    if (!hasStorage()) return
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
    if (!hasStorage()) return
    Object.keys(localStorage)
      .filter(k => k.startsWith(PREFIX))
      .forEach(k => localStorage.removeItem(k))
  },
  getSessionIds(): string[] {
    if (!hasStorage()) return []
    try {
      return JSON.parse(localStorage.getItem(`${PREFIX}sessions`) ?? '[]')
    } catch {
      return []
    }
  },
}

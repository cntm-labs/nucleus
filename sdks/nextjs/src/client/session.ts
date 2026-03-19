const NUCLEUS_SESSION_COOKIE = '__nucleus_session'
const NUCLEUS_REFRESH_COOKIE = '__nucleus_refresh'

export function getSessionToken(): string | null {
  if (typeof document === 'undefined') return null
  const match = document.cookie.match(new RegExp(`(?:^|; )${NUCLEUS_SESSION_COOKIE}=([^;]*)`))
  return match ? decodeURIComponent(match[1]) : null
}

export function setSessionToken(token: string, expiresAt: string): void {
  if (typeof document === 'undefined') return
  const expires = new Date(expiresAt).toUTCString()
  document.cookie = `${NUCLEUS_SESSION_COOKIE}=${encodeURIComponent(token)}; path=/; expires=${expires}; SameSite=Lax; Secure`
}

export function clearSessionToken(): void {
  if (typeof document === 'undefined') return
  document.cookie = `${NUCLEUS_SESSION_COOKIE}=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT`
  document.cookie = `${NUCLEUS_REFRESH_COOKIE}=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT`
}

export function isTokenExpired(expiresAt: string): boolean {
  return new Date(expiresAt).getTime() <= Date.now()
}

export function shouldRefreshToken(expiresAt: string, bufferMs = 60_000): boolean {
  return new Date(expiresAt).getTime() - bufferMs <= Date.now()
}

let refreshPromise: Promise<string | null> | null = null

export async function autoRefresh(
  expiresAt: string,
  refreshFn: () => Promise<{ token: string; expires_at: string }>,
): Promise<string | null> {
  if (!shouldRefreshToken(expiresAt)) {
    return getSessionToken()
  }

  if (refreshPromise) return refreshPromise

  refreshPromise = (async () => {
    try {
      const { token, expires_at } = await refreshFn()
      setSessionToken(token, expires_at)
      return token
    } catch {
      clearSessionToken()
      return null
    } finally {
      refreshPromise = null
    }
  })()

  return refreshPromise
}

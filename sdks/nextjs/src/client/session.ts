const NUCLEUS_SESSION_COOKIE = '__nucleus_session'
const NUCLEUS_REFRESH_COOKIE = '__nucleus_refresh'
const NUCLEUS_EXPIRES_COOKIE = '__nucleus_expires'

export function getSessionToken(): string | null {
  if (typeof document === 'undefined') return null
  const match = document.cookie.match(new RegExp(`(?:^|; )${NUCLEUS_SESSION_COOKIE}=([^;]*)`))
  return match ? decodeURIComponent(match[1]) : null
}

export function getRefreshToken(): string | null {
  if (typeof document === 'undefined') return null
  const match = document.cookie.match(new RegExp(`(?:^|; )${NUCLEUS_REFRESH_COOKIE}=([^;]*)`))
  return match ? decodeURIComponent(match[1]) : null
}

export function getExpiresAt(): string | null {
  if (typeof document === 'undefined') return null
  const match = document.cookie.match(new RegExp(`(?:^|; )${NUCLEUS_EXPIRES_COOKIE}=([^;]*)`))
  return match ? decodeURIComponent(match[1]) : null
}

export function setSessionTokens(token: string, refreshToken: string, expiresAt: string): void {
  if (typeof document === 'undefined') return
  const expires = new Date(expiresAt).toUTCString()
  // Note: In production, the server should set HttpOnly cookies via Set-Cookie headers.
  // Client-side cookie setting is used here as a fallback for SPA-mode sign-in flows.
  // For maximum security, use a Next.js API route or Server Action to proxy auth and set HttpOnly cookies.
  document.cookie = `${NUCLEUS_SESSION_COOKIE}=${encodeURIComponent(token)}; path=/; expires=${expires}; SameSite=Lax; Secure`
  document.cookie = `${NUCLEUS_REFRESH_COOKIE}=${encodeURIComponent(refreshToken)}; path=/; expires=${expires}; SameSite=Lax; Secure`
  document.cookie = `${NUCLEUS_EXPIRES_COOKIE}=${encodeURIComponent(expiresAt)}; path=/; expires=${expires}; SameSite=Lax; Secure`
}

export function clearSessionTokens(): void {
  if (typeof document === 'undefined') return
  const clear = (name: string) => { document.cookie = `${name}=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT` }
  clear(NUCLEUS_SESSION_COOKIE)
  clear(NUCLEUS_REFRESH_COOKIE)
  clear(NUCLEUS_EXPIRES_COOKIE)
}

export function isTokenExpired(expiresAt: string): boolean {
  return new Date(expiresAt).getTime() <= Date.now()
}

export function shouldRefreshToken(expiresAt: string, bufferMs = 60_000): boolean {
  const expiresMs = new Date(expiresAt).getTime()
  if (isNaN(expiresMs)) return false
  return expiresMs - bufferMs <= Date.now()
}

let refreshPromise: Promise<string | null> | null = null

export async function autoRefresh(
  expiresAt: string,
  refreshFn: () => Promise<{ token: string; refresh_token: string; expires_at: string }>,
): Promise<string | null> {
  if (!shouldRefreshToken(expiresAt)) {
    return getSessionToken()
  }

  if (refreshPromise) return refreshPromise

  refreshPromise = (async () => {
    try {
      const { token, refresh_token, expires_at } = await refreshFn()
      setSessionTokens(token, refresh_token, expires_at)
      return token
    } catch {
      clearSessionTokens()
      return null
    } finally {
      refreshPromise = null
    }
  })()

  return refreshPromise
}

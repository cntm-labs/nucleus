import { describe, it, expect, beforeEach } from 'vitest'
import { getSessionToken, getRefreshToken, getExpiresAt, setSessionTokens, clearSessionTokens, shouldRefreshToken } from '../src/client/session'

describe('cookie session', () => {
  beforeEach(() => {
    // Clear cookies by expiring them
    document.cookie = '__nucleus_session=; expires=Thu, 01 Jan 1970 00:00:00 GMT'
    document.cookie = '__nucleus_refresh=; expires=Thu, 01 Jan 1970 00:00:00 GMT'
    document.cookie = '__nucleus_expires=; expires=Thu, 01 Jan 1970 00:00:00 GMT'
  })

  it('round-trip via document.cookie works', () => {
    document.cookie = '__nucleus_session=tok'
    document.cookie = '__nucleus_refresh=ref'
    document.cookie = '__nucleus_expires=2030'
    expect(getSessionToken()).toBe('tok')
    expect(getRefreshToken()).toBe('ref')
    expect(getExpiresAt()).toBe('2030')
  })

  it('getSessionToken reads from cookie', () => {
    Object.defineProperty(document, 'cookie', { value: '__nucleus_session=my_token; other=val', writable: true, configurable: true })
    expect(getSessionToken()).toBe('my_token')
  })

  it('getRefreshToken reads from cookie', () => {
    Object.defineProperty(document, 'cookie', { value: '__nucleus_refresh=my_ref', writable: true, configurable: true })
    expect(getRefreshToken()).toBe('my_ref')
  })

  it('getExpiresAt reads from cookie', () => {
    Object.defineProperty(document, 'cookie', { value: '__nucleus_expires=2030-01-01', writable: true, configurable: true })
    expect(getExpiresAt()).toBe('2030-01-01')
  })

  it('getSessionToken returns null when no cookie', () => {
    expect(getSessionToken()).toBeNull()
  })

  it('shouldRefreshToken returns false for NaN date', () => {
    expect(shouldRefreshToken('')).toBe(false)
  })

  it('shouldRefreshToken returns true for near-expiry', () => {
    const soon = new Date(Date.now() + 30000).toISOString()
    expect(shouldRefreshToken(soon)).toBe(true)
  })

  it('shouldRefreshToken returns false for far future', () => {
    const far = new Date(Date.now() + 600000).toISOString()
    expect(shouldRefreshToken(far)).toBe(false)
  })
})

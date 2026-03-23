import { describe, it, expect, beforeEach } from 'vitest'
import { TokenStorage } from '../src/storage'

const store = new Map<string, string>()

// Mock localStorage
Object.defineProperty(globalThis, 'localStorage', {
  value: {
    getItem: (key: string) => store.get(key) ?? null,
    setItem: (key: string, value: string) => store.set(key, value),
    removeItem: (key: string) => store.delete(key),
  },
  writable: true,
})

// Needed for clearAll which calls Object.keys(localStorage)
Object.keys = ((orig) => (obj: object) => {
  if (obj === localStorage) return [...store.keys()]
  return orig(obj)
})(Object.keys)

describe('TokenStorage', () => {
  beforeEach(() => store.clear())

  describe('setSession', () => {
    it('stores token, refresh_token, expires_at, and session_id', () => {
      TokenStorage.setSession({ id: 's_1', token: 'tok', refresh_token: 'ref', expires_at: '2025-01-01' })
      expect(TokenStorage.getActiveToken()).toBe('tok')
      expect(TokenStorage.getRefreshToken()).toBe('ref')
      expect(TokenStorage.getExpiresAt()).toBe('2025-01-01')
      expect(TokenStorage.getActiveSessionId()).toBe('s_1')
    })

    it('adds session to session list without duplicates', () => {
      TokenStorage.setSession({ id: 's_1', token: 'a', refresh_token: 'b', expires_at: 'c' })
      TokenStorage.setSession({ id: 's_1', token: 'a2', refresh_token: 'b2', expires_at: 'c2' })
      expect(TokenStorage.getSessionIds()).toEqual(['s_1'])
    })

    it('tracks multiple sessions', () => {
      TokenStorage.setSession({ id: 's_1', token: 'a', refresh_token: 'b', expires_at: 'c' })
      TokenStorage.setSession({ id: 's_2', token: 'x', refresh_token: 'y', expires_at: 'z' })
      expect(TokenStorage.getSessionIds()).toEqual(['s_1', 's_2'])
    })
  })

  describe('clear', () => {
    it('removes active session but keeps session list updated', () => {
      TokenStorage.setSession({ id: 's_1', token: 'a', refresh_token: 'b', expires_at: 'c' })
      TokenStorage.setSession({ id: 's_2', token: 'x', refresh_token: 'y', expires_at: 'z' })
      TokenStorage.clear()
      expect(TokenStorage.getActiveToken()).toBeNull()
      expect(TokenStorage.getSessionIds()).toEqual(['s_1'])
    })
  })

  describe('clearAll', () => {
    it('removes all nucleus keys', () => {
      TokenStorage.setSession({ id: 's_1', token: 'a', refresh_token: 'b', expires_at: 'c' })
      store.set('other_key', 'preserved')
      TokenStorage.clearAll()
      expect(TokenStorage.getActiveToken()).toBeNull()
      expect(TokenStorage.getSessionIds()).toEqual([])
      expect(store.get('other_key')).toBe('preserved')
    })
  })

  describe('getSessionIds', () => {
    it('returns empty array on invalid JSON', () => {
      store.set('__nucleus_sessions', 'not json')
      expect(TokenStorage.getSessionIds()).toEqual([])
    })

    it('returns empty array when key missing', () => {
      expect(TokenStorage.getSessionIds()).toEqual([])
    })
  })
})

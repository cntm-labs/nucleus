import { describe, it, expect, vi, beforeEach } from 'vitest'
import { SessionManager } from '../src/client/session'
import type { NucleusApi } from '../src/client/api'

const store = new Map<string, string>()
Object.defineProperty(globalThis, 'localStorage', {
  value: {
    getItem: (key: string) => store.get(key) ?? null,
    setItem: (key: string, value: string) => store.set(key, value),
    removeItem: (key: string) => store.delete(key),
  },
  writable: true,
})

describe('SessionManager', () => {
  let api: NucleusApi
  let onChange: ReturnType<typeof vi.fn>
  let manager: SessionManager

  beforeEach(() => {
    vi.clearAllMocks()
    store.clear()
    api = { refreshSession: vi.fn() } as unknown as NucleusApi
    onChange = vi.fn()
    manager = new SessionManager(api, onChange)
  })

  it('restore returns null when no stored tokens', () => {
    expect(manager.restore()).toBeNull()
  })

  it('restore returns stored data when tokens exist', () => {
    store.set('__nucleus_token', 'tok')
    store.set('__nucleus_refresh_token', 'ref')
    store.set('__nucleus_expires_at', '2030-01-01')
    store.set('__nucleus_session_id', 's_1')
    const result = manager.restore()
    expect(result).not.toBeNull()
    expect(result!.token).toBe('tok')
    expect(result!.refresh_token).toBe('ref')
  })

  it('setSession stores to localStorage and notifies', () => {
    const session = { id: 's_1', token: 'tok', refresh_token: 'ref', expires_at: new Date(Date.now() + 300000).toISOString(), user_id: '' }
    manager.setSession(session as any)
    expect(store.get('__nucleus_token')).toBe('tok')
    expect(onChange).toHaveBeenCalled()
  })

  it('clearSession removes from storage and notifies null', () => {
    store.set('__nucleus_token', 'tok')
    store.set('__nucleus_session_id', 's_1')
    store.set('__nucleus_sessions', '["s_1"]')
    manager.clearSession()
    expect(store.get('__nucleus_token')).toBeUndefined()
    expect(onChange).toHaveBeenCalledWith(null)
  })

  it('getSessionIds returns parsed array', () => {
    store.set('__nucleus_sessions', '["s_1","s_2"]')
    expect(manager.getSessionIds()).toEqual(['s_1', 's_2'])
  })

  it('getSessionIds returns empty on invalid JSON', () => {
    store.set('__nucleus_sessions', 'bad')
    expect(manager.getSessionIds()).toEqual([])
  })

  it('refresh clears session on failure', async () => {
    store.set('__nucleus_refresh_token', 'ref')
    vi.mocked(api.refreshSession).mockRejectedValue(new Error('fail'))
    await manager.refresh()
    expect(onChange).toHaveBeenCalledWith(null)
  })

  it('destroy cancels refresh timer', () => {
    manager.destroy()
    // Should not throw
  })
})

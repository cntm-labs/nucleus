import { describe, it, expect, vi, beforeEach } from 'vitest'
import { SessionManager } from '../src/session'
import { TokenStorage } from '../src/storage'
import type { NucleusApi } from '../src/api'

vi.mock('../src/storage', () => ({
  TokenStorage: {
    getActiveToken: vi.fn(),
    getRefreshToken: vi.fn(),
    getExpiresAt: vi.fn(),
    getActiveSessionId: vi.fn(),
    setSession: vi.fn(),
    clear: vi.fn(),
    getSessionIds: vi.fn(() => []),
  },
}))

function createMockApi(): NucleusApi {
  return {
    refreshSession: vi.fn(),
  } as unknown as NucleusApi
}

describe('SessionManager', () => {
  let api: NucleusApi
  let onChange: ReturnType<typeof vi.fn>
  let manager: SessionManager

  beforeEach(() => {
    vi.clearAllMocks()
    vi.useRealTimers()
    api = createMockApi()
    onChange = vi.fn()
    manager = new SessionManager(api, onChange)
  })

  describe('restore', () => {
    it('returns null when no stored tokens', async () => {
      vi.mocked(TokenStorage.getActiveToken).mockReturnValue(null)
      const result = await manager.restore()
      expect(result).toBeNull()
    })

    it('returns session when valid tokens exist and not expired', async () => {
      vi.mocked(TokenStorage.getActiveToken).mockReturnValue('tok')
      vi.mocked(TokenStorage.getRefreshToken).mockReturnValue('ref')
      vi.mocked(TokenStorage.getExpiresAt).mockReturnValue(new Date(Date.now() + 300000).toISOString())
      vi.mocked(TokenStorage.getActiveSessionId).mockReturnValue('s_1')

      const result = await manager.restore()
      expect(result).not.toBeNull()
      expect(result!.token).toBe('tok')
    })

    it('calls refresh when token is expired', async () => {
      vi.mocked(TokenStorage.getActiveToken).mockReturnValue('old_tok')
      vi.mocked(TokenStorage.getRefreshToken).mockReturnValue('ref')
      vi.mocked(TokenStorage.getExpiresAt).mockReturnValue('2020-01-01T00:00:00Z')
      vi.mocked(TokenStorage.getActiveSessionId).mockReturnValue('s_1')

      const newSession = { id: 's_2', token: 'new', refresh_token: 'new_ref', expires_at: '2030-01-01T00:00:00Z', user_id: '' }
      vi.mocked(api.refreshSession).mockResolvedValue(newSession as any)

      const result = await manager.restore()
      expect(api.refreshSession).toHaveBeenCalledWith('ref')
    })
  })

  describe('setSession', () => {
    it('stores session and notifies onChange', () => {
      const session = { id: 's_1', token: 'tok', refresh_token: 'ref', expires_at: new Date(Date.now() + 300000).toISOString(), user_id: '' }
      manager.setSession(session as any)
      expect(TokenStorage.setSession).toHaveBeenCalled()
      expect(onChange).toHaveBeenCalledWith(session)
    })
  })

  describe('clearSession', () => {
    it('clears storage and notifies with null', () => {
      manager.clearSession()
      expect(TokenStorage.clear).toHaveBeenCalled()
      expect(onChange).toHaveBeenCalledWith(null)
    })
  })

  describe('refresh', () => {
    it('clears session on refresh failure', async () => {
      vi.mocked(TokenStorage.getRefreshToken).mockReturnValue('ref')
      vi.mocked(api.refreshSession).mockRejectedValue(new Error('fail'))

      const result = await manager.refresh()
      expect(result).toBeNull()
      expect(onChange).toHaveBeenCalledWith(null)
    })
  })

  describe('destroy', () => {
    it('cancels refresh timer', () => {
      vi.useFakeTimers()
      const session = { id: 's_1', token: 'tok', refresh_token: 'ref', expires_at: new Date(Date.now() + 300000).toISOString(), user_id: '' }
      manager.setSession(session as any)
      manager.destroy()
      // Should not throw
      vi.advanceTimersByTime(400000)
      vi.useRealTimers()
    })
  })
})

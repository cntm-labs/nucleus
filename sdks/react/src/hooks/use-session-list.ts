import { useState, useCallback } from 'react'
import { useNucleus } from '../provider'
import type { NucleusSession } from '../client/types'

export function useSessionList() {
  const { _api, _sessionManager, _setUser, getToken } = useNucleus()
  const [sessions, setSessions] = useState<NucleusSession[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const fetchSessions = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const list = await _api.getSessions(getToken()!)
      setSessions(list)
      return list
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load sessions')
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [_api, getToken])

  const revokeSession = useCallback(async (sessionId: string) => {
    setError(null)
    try {
      await _api.revokeSession(getToken()!, sessionId)
      setSessions(prev => prev.filter(s => s.id !== sessionId))
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to revoke session')
      throw err
    }
  }, [_api, getToken])

  const switchSession = useCallback(async (sessionId: string) => {
    setError(null)
    try {
      const session = await _api.switchSession(getToken()!, sessionId)
      _sessionManager.setSession(session)
      const user = await _api.getUser(session.token)
      _setUser(user)
      return session
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to switch session')
      throw err
    }
  }, [_api, _sessionManager, _setUser, getToken])

  return { sessions, fetchSessions, revokeSession, switchSession, isLoading, error }
}

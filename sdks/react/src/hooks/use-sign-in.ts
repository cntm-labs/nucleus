import { useState, useCallback } from 'react'
import { useNucleus } from '../provider'

export function useSignIn() {
  const { _api, _sessionManager, _setUser } = useNucleus()
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const signIn = useCallback(async (email: string, password: string) => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await _api.signIn(email, password)
      _setUser(result.user)
      _sessionManager.setSession(result.session)
      return result
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Sign in failed'
      setError(message)
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [_api, _sessionManager, _setUser])

  return { signIn, isLoading, error }
}

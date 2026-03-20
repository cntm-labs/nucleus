import { useState, useCallback } from 'react'
import { useNucleus } from '../provider'

export function useSignUp() {
  const { _api, _sessionManager, _setUser } = useNucleus()
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const signUp = useCallback(async (email: string, password: string, firstName?: string, lastName?: string) => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await _api.signUp(email, password, firstName, lastName)
      _setUser(result.user)
      _sessionManager.setSession(result.session)
      return result
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Sign up failed'
      setError(message)
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [_api, _sessionManager, _setUser])

  return { signUp, isLoading, error }
}

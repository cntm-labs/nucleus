import { useState, useCallback } from 'react'
import { useNucleus } from '../provider'
import { setSessionToken } from '../client/session'

export function useSignIn() {
  const { _api, _setUser, _setSession } = useNucleus()
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const signIn = useCallback(async (email: string, password: string) => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await _api.signIn(email, password)
      _setUser(result.user)
      _setSession(result.session)
      setSessionToken(result.session.token, result.session.expires_at)
      return result
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Sign in failed'
      setError(message)
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [_api, _setUser, _setSession])

  return { signIn, isLoading, error }
}

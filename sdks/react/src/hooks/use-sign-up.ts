import { useState, useCallback } from 'react'
import { useNucleus } from '../provider'

interface SignUpAttempt {
  status: 'idle' | 'loading' | 'success' | 'error'
  error: string | null
}

export function useSignUp() {
  const { _api, _setUser, _setSession } = useNucleus()
  const [attempt, setAttempt] = useState<SignUpAttempt>({ status: 'idle', error: null })

  const signUp = useCallback(async (email: string, password: string, firstName?: string, lastName?: string) => {
    setAttempt({ status: 'loading', error: null })
    try {
      const result = await _api.signUp(email, password, firstName, lastName)
      _setUser(result.user)
      _setSession(result.session)
      if (typeof window !== 'undefined') {
        window.localStorage.setItem('__nucleus_session_token', result.session.token)
      }
      setAttempt({ status: 'success', error: null })
      return result
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Sign up failed'
      setAttempt({ status: 'error', error: message })
      throw err
    }
  }, [_api, _setUser, _setSession])

  return {
    signUp,
    isLoading: attempt.status === 'loading',
    error: attempt.error,
  }
}

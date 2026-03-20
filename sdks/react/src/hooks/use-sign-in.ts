import { useState, useCallback } from 'react'
import { useNucleus } from '../provider'

interface SignInAttempt {
  status: 'idle' | 'loading' | 'success' | 'error'
  error: string | null
}

export function useSignIn() {
  const { _api, _setUser, _setSession } = useNucleus()
  const [attempt, setAttempt] = useState<SignInAttempt>({ status: 'idle', error: null })

  const signIn = useCallback(async (email: string, password: string) => {
    setAttempt({ status: 'loading', error: null })
    try {
      const result = await _api.signIn(email, password)
      _setUser(result.user)
      _setSession(result.session)
      if (typeof window !== 'undefined') {
        window.localStorage.setItem('__nucleus_session_token', result.session.token)
      }
      setAttempt({ status: 'success', error: null })
      return result
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Sign in failed'
      setAttempt({ status: 'error', error: message })
      throw err
    }
  }, [_api, _setUser, _setSession])

  return {
    signIn,
    isLoading: attempt.status === 'loading',
    error: attempt.error,
  }
}

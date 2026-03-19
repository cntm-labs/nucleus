import { useState, useCallback } from 'react'

interface SignInAttempt {
  status: 'idle' | 'loading' | 'success' | 'error'
  error: string | null
}

export function useSignIn() {
  const [attempt, setAttempt] = useState<SignInAttempt>({ status: 'idle', error: null })

  const signIn = useCallback(async (_email: string, _password: string) => {
    setAttempt({ status: 'loading', error: null })
    try {
      // TODO: call NucleusApi.signIn and update context
      setAttempt({ status: 'success', error: null })
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Sign in failed'
      setAttempt({ status: 'error', error: message })
    }
  }, [])

  return {
    signIn,
    isLoading: attempt.status === 'loading',
    error: attempt.error,
  }
}

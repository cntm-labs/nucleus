import { useState, useCallback } from 'react'

interface SignUpAttempt {
  status: 'idle' | 'loading' | 'success' | 'error'
  error: string | null
}

export function useSignUp() {
  const [attempt, setAttempt] = useState<SignUpAttempt>({ status: 'idle', error: null })

  const signUp = useCallback(async (_email: string, _password: string, _firstName?: string, _lastName?: string) => {
    setAttempt({ status: 'loading', error: null })
    try {
      // TODO: call NucleusApi.signUp and update context
      setAttempt({ status: 'success', error: null })
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Sign up failed'
      setAttempt({ status: 'error', error: message })
    }
  }, [])

  return {
    signUp,
    isLoading: attempt.status === 'loading',
    error: attempt.error,
  }
}

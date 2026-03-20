import { useState, useCallback } from 'react'
import { useNucleus } from '../provider'

export function useVerification() {
  const { _api, getToken } = useNucleus()
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const sendEmailVerification = useCallback(async (email: string) => {
    setIsLoading(true)
    setError(null)
    try {
      return await _api.sendEmailVerification(getToken()!, email)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to send email verification')
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [_api, getToken])

  const confirmEmailVerification = useCallback(async (code: string) => {
    setIsLoading(true)
    setError(null)
    try {
      return await _api.confirmEmailVerification(getToken()!, code)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Email verification failed')
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [_api, getToken])

  const sendPhoneVerification = useCallback(async (phone: string) => {
    setIsLoading(true)
    setError(null)
    try {
      return await _api.sendPhoneVerification(getToken()!, phone)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to send phone verification')
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [_api, getToken])

  const confirmPhoneVerification = useCallback(async (code: string) => {
    setIsLoading(true)
    setError(null)
    try {
      return await _api.confirmPhoneVerification(getToken()!, code)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Phone verification failed')
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [_api, getToken])

  return { sendEmailVerification, confirmEmailVerification, sendPhoneVerification, confirmPhoneVerification, isLoading, error }
}

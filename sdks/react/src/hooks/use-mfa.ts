import { useState, useCallback } from 'react'
import { useNucleus } from '../provider'
import type { NucleusMfaSetup } from '../client/types'

export function useMfa() {
  const { _api, getToken } = useNucleus()
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const setupTotp = useCallback(async (): Promise<NucleusMfaSetup> => {
    setIsLoading(true)
    setError(null)
    try {
      return await _api.mfaTotpSetup(getToken()!)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'MFA setup failed')
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [_api, getToken])

  const verifyTotp = useCallback(async (code: string) => {
    setIsLoading(true)
    setError(null)
    try {
      return await _api.mfaTotpVerify(getToken()!, code)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'MFA verification failed')
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [_api, getToken])

  const sendSms = useCallback(async (phone: string) => {
    setIsLoading(true)
    setError(null)
    try {
      return await _api.mfaSmsSend(getToken()!, phone)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'SMS send failed')
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [_api, getToken])

  const verifySms = useCallback(async (code: string) => {
    setIsLoading(true)
    setError(null)
    try {
      return await _api.mfaSmsVerify(getToken()!, code)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'SMS verification failed')
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [_api, getToken])

  const getBackupCodes = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      return await _api.mfaBackupCodes(getToken()!)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to get backup codes')
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [_api, getToken])

  return { setupTotp, verifyTotp, sendSms, verifySms, getBackupCodes, isLoading, error }
}

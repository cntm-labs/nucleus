import { useState, useCallback } from 'react'
import { useNucleus } from '../provider'

export function useProfile() {
  const { _api, _setUser, getToken } = useNucleus()
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const updateProfile = useCallback(async (data: { first_name?: string; last_name?: string; avatar_url?: string }) => {
    setIsLoading(true)
    setError(null)
    try {
      const user = await _api.updateUser(getToken()!, data)
      _setUser(user)
      return user
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Profile update failed')
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [_api, _setUser, getToken])

  const updatePassword = useCallback(async (currentPassword: string, newPassword: string) => {
    setIsLoading(true)
    setError(null)
    try {
      return await _api.updatePassword(getToken()!, currentPassword, newPassword)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Password update failed')
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [_api, getToken])

  const updateEmail = useCallback(async (email: string) => {
    setIsLoading(true)
    setError(null)
    try {
      return await _api.updateEmail(getToken()!, email)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Email update failed')
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [_api, getToken])

  return { updateProfile, updatePassword, updateEmail, isLoading, error }
}

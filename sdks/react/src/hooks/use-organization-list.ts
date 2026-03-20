import { useState, useCallback } from 'react'
import { useNucleus } from '../provider'
import type { NucleusOrganization } from '../client/types'

export function useOrganizationList() {
  const { _api, _setOrganization, getToken } = useNucleus()
  const [organizations, setOrganizations] = useState<NucleusOrganization[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const fetchOrganizations = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const list = await _api.getOrganizations(getToken()!)
      setOrganizations(list)
      return list
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load organizations')
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [_api, getToken])

  const createOrganization = useCallback(async (name: string, slug: string) => {
    setError(null)
    try {
      const org = await _api.createOrganization(getToken()!, name, slug)
      setOrganizations(prev => [...prev, org])
      return org
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create organization')
      throw err
    }
  }, [_api, getToken])

  const setActiveOrganization = useCallback((org: NucleusOrganization | null) => {
    _setOrganization(org)
  }, [_setOrganization])

  return { organizations, fetchOrganizations, createOrganization, setActiveOrganization, isLoading, error }
}

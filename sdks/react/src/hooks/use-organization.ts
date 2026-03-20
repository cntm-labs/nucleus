import { useNucleus } from '../provider'

export function useOrganization() {
  const { organization, isLoaded } = useNucleus()
  return {
    organization,
    isLoaded,
    isActive: !!organization,
  }
}

import { useNucleus } from '../provider'

export function useSession() {
  const { session, isLoaded } = useNucleus()
  return { session, isLoaded, isActive: !!session }
}

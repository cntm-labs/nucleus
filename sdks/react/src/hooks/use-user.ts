import { useNucleus } from '../provider'

export function useUser() {
  const { user, isLoaded, isSignedIn } = useNucleus()
  return { user, isLoaded, isSignedIn }
}

import { useNucleus } from '../provider'

export function useAuth() {
  const { isLoaded, isSignedIn, user, signOut, getToken } = useNucleus()
  return {
    isLoaded,
    isSignedIn,
    userId: user?.id ?? null,
    signOut,
    getToken,
  }
}

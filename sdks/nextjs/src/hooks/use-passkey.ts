import { useState, useCallback } from 'react'
import { useNucleus } from '../provider'
import { setSessionTokens } from '../client/session'

export function usePasskey() {
  const { _api, _setUser, _setSession, getToken } = useNucleus()
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const signInWithPasskey = useCallback(async () => {
    const token = getToken()
    if (!token) throw new Error('Must be signed in to use passkeys')
    setIsLoading(true)
    setError(null)
    try {
      const challenge = await _api.getPasskeyChallenge(token)
      const credential = await navigator.credentials.get({
        publicKey: {
          challenge: Uint8Array.from(atob(challenge.challenge as unknown as string), c => c.charCodeAt(0)),
          rpId: window.location.hostname,
          userVerification: 'preferred',
        },
      }) as PublicKeyCredential
      const response = credential.response as AuthenticatorAssertionResponse
      const result = await _api.verifyPasskey({
        id: credential.id,
        rawId: btoa(String.fromCharCode(...new Uint8Array(credential.rawId))),
        response: {
          authenticatorData: btoa(String.fromCharCode(...new Uint8Array(response.authenticatorData))),
          clientDataJSON: btoa(String.fromCharCode(...new Uint8Array(response.clientDataJSON))),
          signature: btoa(String.fromCharCode(...new Uint8Array(response.signature))),
        },
        type: credential.type,
      })
      _setUser(result.user)
      _setSession(result.session)
      setSessionTokens(result.session.token, result.session.refresh_token, result.session.expires_at)
      return result
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Passkey sign in failed')
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [_api, _setUser, _setSession, getToken])

  return { signInWithPasskey, isLoading, error }
}

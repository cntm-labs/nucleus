import type { NucleusAuthResponse } from './types'
import type { NucleusApi } from './api'

export async function signInWithPasskey(api: NucleusApi, token: string): Promise<NucleusAuthResponse> {
  if (typeof window === 'undefined') {
    throw new Error('Passkeys are only available in browser environments')
  }
  const challenge = await api.getPasskeyChallenge(token)
  const credential = await navigator.credentials.get({
    publicKey: {
      challenge: Uint8Array.from(atob(challenge.challenge as unknown as string), c => c.charCodeAt(0)),
      rpId: window.location.hostname,
      userVerification: 'preferred',
    },
  }) as PublicKeyCredential
  const response = credential.response as AuthenticatorAssertionResponse
  return api.verifyPasskey({
    id: credential.id,
    rawId: btoa(String.fromCharCode(...new Uint8Array(credential.rawId))),
    response: {
      authenticatorData: btoa(String.fromCharCode(...new Uint8Array(response.authenticatorData))),
      clientDataJSON: btoa(String.fromCharCode(...new Uint8Array(response.clientDataJSON))),
      signature: btoa(String.fromCharCode(...new Uint8Array(response.signature))),
    },
    type: credential.type,
  })
}

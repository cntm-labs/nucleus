import { useState, useCallback } from 'react'
import { useNucleus } from '../provider'
import { setSessionTokens } from '../client/session'
import type { OAuthProvider, NucleusAuthResponse } from '../client/types'

export function useOAuth() {
  const { _api, _setUser, _setSession } = useNucleus()
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const signInWithOAuth = useCallback(async (provider: OAuthProvider): Promise<NucleusAuthResponse> => {
    setIsLoading(true)
    setError(null)
    try {
      const redirectUri = `${window.location.origin}/__nucleus/oauth/callback`
      const state = crypto.randomUUID()
      sessionStorage.setItem('__nucleus_oauth_state', state)
      const url = _api.getOAuthUrl(provider, redirectUri, state)
      const popup = window.open(url, 'nucleus-oauth', 'width=500,height=700,popup=true')
      if (!popup) throw new Error('Failed to open OAuth popup')

      const result = await new Promise<NucleusAuthResponse>((resolve, reject) => {
        const onMessage = async (event: MessageEvent) => {
          if (event.origin !== window.location.origin) return
          if (event.data?.type !== 'nucleus:oauth:callback') return
          window.removeEventListener('message', onMessage)
          if (event.data.state !== sessionStorage.getItem('__nucleus_oauth_state')) {
            reject(new Error('OAuth state mismatch — possible CSRF attack'))
            return
          }
          sessionStorage.removeItem('__nucleus_oauth_state')
          const { code, error: oauthError } = event.data
          if (oauthError) { reject(new Error(oauthError)); return }
          try { resolve(await _api.exchangeOAuthCode(code, redirectUri)) } catch (e) { reject(e) }
        }
        window.addEventListener('message', onMessage)
        const poll = setInterval(() => {
          if (popup.closed) { clearInterval(poll); window.removeEventListener('message', onMessage); reject(new Error('OAuth popup closed')) }
        }, 500)
      })

      _setUser(result.user)
      _setSession(result.session)
      setSessionTokens(result.session.token, result.session.refresh_token, result.session.expires_at)
      return result
    } catch (err) {
      setError(err instanceof Error ? err.message : 'OAuth sign in failed')
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [_api, _setUser, _setSession])

  return { signInWithOAuth, isLoading, error }
}

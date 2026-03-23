import type { NucleusAuthResponse, OAuthProvider } from './types'
import type { NucleusApi } from './api'

export function startOAuthFlow(
  api: NucleusApi,
  provider: OAuthProvider,
): Promise<NucleusAuthResponse> {
  if (typeof window === 'undefined') {
    return Promise.reject(new Error('OAuth is only available in browser environments'))
  }
  return new Promise((resolve, reject) => {
    const redirectUri = `${window.location.origin}/__nucleus/oauth/callback`
    const state = crypto.randomUUID()
    sessionStorage.setItem('__nucleus_oauth_state', state)
    const url = api.getOAuthUrl(provider, redirectUri, state)
    const popup = window.open(url, 'nucleus-oauth', 'width=500,height=700,popup=true')
    if (!popup) { reject(new Error('Failed to open OAuth popup')); return }

    const onMessage = async (event: MessageEvent) => {
      if (event.origin !== window.location.origin) return
      if (event.data?.type !== 'nucleus:oauth:callback') return
      window.removeEventListener('message', onMessage)
      if (event.data.state !== sessionStorage.getItem('__nucleus_oauth_state')) {
        reject(new Error('OAuth state mismatch — possible CSRF attack'))
        return
      }
      sessionStorage.removeItem('__nucleus_oauth_state')
      const { code, error } = event.data
      if (error) { reject(new Error(error)); return }
      try {
        const result = await api.exchangeOAuthCode(code, redirectUri)
        resolve(result)
      } catch (e) { reject(e) }
    }
    window.addEventListener('message', onMessage)

    // Poll for popup close
    const poll = setInterval(() => {
      if (popup.closed) { clearInterval(poll); window.removeEventListener('message', onMessage); reject(new Error('OAuth popup closed')) }
    }, 500)
  })
}

import React, { useState } from 'react'
import { useSignIn } from '../hooks/use-sign-in'
import { useOAuth } from '../hooks/use-oauth'
import { useStyles, Divider } from './appearance'
import type { OAuthProvider } from '../client/types'

export interface SignInProps {
  afterSignInUrl?: string
  routing?: 'hash' | 'path'
  onSignIn?: () => void
  oauthProviders?: OAuthProvider[]
  showPasskey?: boolean
}

export function SignIn({ afterSignInUrl = '/', onSignIn, oauthProviders = [] }: SignInProps) {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [mfaCode, setMfaCode] = useState('')
  const [step, setStep] = useState<'credentials' | 'mfa'>('credentials')
  const { signIn, isLoading, error } = useSignIn()
  const { signInWithOAuth, isLoading: oauthLoading, error: oauthError } = useOAuth()
  const s = useStyles()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await signIn(email, password)
      onSignIn?.()
      if (typeof window !== 'undefined') window.location.href = afterSignInUrl
    } catch (err) {
      if (err instanceof Error && err.message.includes('mfa_required')) {
        setStep('mfa')
      }
    }
  }

  const handleOAuth = async (provider: OAuthProvider) => {
    try {
      await signInWithOAuth(provider)
      onSignIn?.()
      if (typeof window !== 'undefined') window.location.href = afterSignInUrl
    } catch { /* error tracked by hook */ }
  }

  const displayError = error || oauthError

  return (
    <div style={s.card}>
      <h2 style={s.title}>Sign In</h2>
      {displayError && <div style={s.error}>{displayError}</div>}

      {step === 'credentials' && (
        <>
          {oauthProviders.length > 0 && (
            <>
              {oauthProviders.map(provider => (
                <button
                  key={provider}
                  onClick={() => handleOAuth(provider)}
                  disabled={oauthLoading}
                  style={s.secondaryButton}
                >
                  Continue with {provider.charAt(0).toUpperCase() + provider.slice(1)}
                </button>
              ))}
              <Divider text="or" />
            </>
          )}

          <form onSubmit={handleSubmit}>
            <input
              type="email" placeholder="Email" value={email}
              onChange={e => setEmail(e.target.value)}
              style={s.input} required
            />
            <input
              type="password" placeholder="Password" value={password}
              onChange={e => setPassword(e.target.value)}
              style={{ ...s.input, marginBottom: 16 }} required
            />
            <button type="submit" disabled={isLoading} style={{ ...s.button, opacity: isLoading ? 0.7 : 1 }}>
              {isLoading ? 'Signing in...' : 'Sign In'}
            </button>
          </form>
        </>
      )}

      {step === 'mfa' && (
        <form onSubmit={async (e) => {
          e.preventDefault()
          // MFA verification would be handled by useMfa hook in a real flow
          setStep('credentials')
        }}>
          <p style={{ fontSize: 14, marginBottom: 12, color: '#6b7280' }}>
            Enter the verification code from your authenticator app
          </p>
          <input
            type="text" placeholder="000000" value={mfaCode}
            onChange={e => setMfaCode(e.target.value)}
            style={{ ...s.input, textAlign: 'center', letterSpacing: 8, fontSize: 20 }}
            maxLength={6} required
          />
          <button type="submit" style={{ ...s.button, marginTop: 8 }}>
            Verify
          </button>
        </form>
      )}
    </div>
  )
}

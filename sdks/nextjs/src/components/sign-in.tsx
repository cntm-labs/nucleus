'use client'
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
}

export function SignIn({ afterSignInUrl = '/', onSignIn, oauthProviders = [] }: SignInProps) {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const { signIn, isLoading, error } = useSignIn()
  const { signInWithOAuth, isLoading: oauthLoading, error: oauthError } = useOAuth()
  const s = useStyles()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await signIn(email, password)
      onSignIn?.()
      if (typeof window !== 'undefined') window.location.href = afterSignInUrl
    } catch { /* error tracked */ }
  }

  const handleOAuth = async (provider: OAuthProvider) => {
    try {
      await signInWithOAuth(provider)
      onSignIn?.()
      if (typeof window !== 'undefined') window.location.href = afterSignInUrl
    } catch { /* error tracked */ }
  }

  const displayError = error || oauthError

  return (
    <div style={s.card}>
      <h2 style={s.title}>Sign In</h2>
      {displayError && <div style={s.error}>{displayError}</div>}
      {oauthProviders.length > 0 && (
        <>
          {oauthProviders.map(provider => (
            <button key={provider} onClick={() => handleOAuth(provider)} disabled={oauthLoading} style={s.secondaryButton}>
              Continue with {provider.charAt(0).toUpperCase() + provider.slice(1)}
            </button>
          ))}
          <Divider text="or" />
        </>
      )}
      <form onSubmit={handleSubmit}>
        <input type="email" placeholder="Email" value={email} onChange={e => setEmail(e.target.value)} style={s.input} required />
        <input type="password" placeholder="Password" value={password} onChange={e => setPassword(e.target.value)} style={{ ...s.input, marginBottom: 16 }} required />
        <button type="submit" disabled={isLoading} style={{ ...s.button, opacity: isLoading ? 0.7 : 1 }}>
          {isLoading ? 'Signing in...' : 'Sign In'}
        </button>
      </form>
    </div>
  )
}

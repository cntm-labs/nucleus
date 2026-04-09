'use client'
import React, { useState } from 'react'
import { useSignUp } from '../hooks/use-sign-up'
import { useOAuth } from '../hooks/use-oauth'
import { useStyles, Divider } from './appearance'
import { useTranslation } from '../i18n'
import type { OAuthProvider } from '../client/types'

export interface SignUpProps {
  afterSignUpUrl?: string
  routing?: 'hash' | 'path'
  onSignUp?: () => void
  oauthProviders?: OAuthProvider[]
}

export function SignUp({ afterSignUpUrl = '/', onSignUp, oauthProviders = [] }: SignUpProps) {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [firstName, setFirstName] = useState('')
  const [lastName, setLastName] = useState('')
  const { signUp, isLoading, error } = useSignUp()
  const { signInWithOAuth, isLoading: oauthLoading, error: oauthError } = useOAuth()
  const s = useStyles()
  const t = useTranslation()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await signUp(email, password, firstName, lastName)
      onSignUp?.()
      if (typeof window !== 'undefined') window.location.href = afterSignUpUrl
    } catch { /* error tracked */ }
  }

  const handleOAuth = async (provider: OAuthProvider) => {
    try {
      await signInWithOAuth(provider)
      onSignUp?.()
      if (typeof window !== 'undefined') window.location.href = afterSignUpUrl
    } catch { /* error tracked */ }
  }

  const displayError = error || oauthError

  return (
    <div style={s.card}>
      <h2 style={s.title}>{t('signUp.title')}</h2>
      {displayError && <div style={s.error}>{displayError}</div>}
      {oauthProviders.length > 0 && (
        <>
          {oauthProviders.map(provider => (
            <button key={provider} onClick={() => handleOAuth(provider)} disabled={oauthLoading} style={s.secondaryButton}>
              {t('signUp.oauth', { provider: provider.charAt(0).toUpperCase() + provider.slice(1) })}
            </button>
          ))}
          <Divider text="or" />
        </>
      )}
      <form onSubmit={handleSubmit}>
        <div style={{ display: 'flex', gap: 8 }}>
          <input type="text" placeholder={t('signUp.firstName')} value={firstName} onChange={e => setFirstName(e.target.value)} style={{ ...s.input, flex: 1 }} />
          <input type="text" placeholder={t('signUp.lastName')} value={lastName} onChange={e => setLastName(e.target.value)} style={{ ...s.input, flex: 1 }} />
        </div>
        <input type="email" placeholder={t('signUp.email')} value={email} onChange={e => setEmail(e.target.value)} style={s.input} required />
        <input type="password" placeholder={t('signUp.password')} value={password} onChange={e => setPassword(e.target.value)} style={{ ...s.input, marginBottom: 16 }} required />
        <button type="submit" disabled={isLoading} style={{ ...s.button, opacity: isLoading ? 0.7 : 1 }}>
          {isLoading ? t('signUp.loading') : t('signUp.button')}
        </button>
      </form>
    </div>
  )
}

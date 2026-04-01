'use client'
import React, { useState } from 'react'
import { useSignIn } from '../hooks/use-sign-in'
import { useOAuth } from '../hooks/use-oauth'
import { useMfa } from '../hooks/use-mfa'
import { useStyles, Divider } from './appearance'
import { useTranslation } from '../i18n'
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
  const [mfaCode, setMfaCode] = useState('')
  const [step, setStep] = useState<'credentials' | 'mfa'>('credentials')
  const { signIn, isLoading, error } = useSignIn()
  const { signInWithOAuth, isLoading: oauthLoading, error: oauthError } = useOAuth()
  const { verifyTotp, isLoading: mfaLoading, error: mfaError } = useMfa()
  const s = useStyles()
  const t = useTranslation()

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
    } catch { /* error tracked */ }
  }

  const displayError = error || oauthError || mfaError

  return (
    <div style={s.card}>
      <h2 style={s.title}>{t('signIn.title')}</h2>
      {displayError && <div style={s.error}>{displayError}</div>}

      {step === 'credentials' && (
        <>
          {oauthProviders.length > 0 && (
            <>
              {oauthProviders.map(provider => (
                <button key={provider} onClick={() => handleOAuth(provider)} disabled={oauthLoading} style={s.secondaryButton}>
                  {t('signIn.oauth', { provider: provider.charAt(0).toUpperCase() + provider.slice(1) })}
                </button>
              ))}
              <Divider text="or" />
            </>
          )}
          <form onSubmit={handleSubmit}>
            <input type="email" placeholder={t('signIn.email')} value={email} onChange={e => setEmail(e.target.value)} style={s.input} required />
            <input type="password" placeholder={t('signIn.password')} value={password} onChange={e => setPassword(e.target.value)} style={{ ...s.input, marginBottom: 16 }} required />
            <button type="submit" disabled={isLoading} style={{ ...s.button, opacity: isLoading ? 0.7 : 1 }}>
              {isLoading ? t('signIn.loading') : t('signIn.button')}
            </button>
          </form>
        </>
      )}

      {step === 'mfa' && (
        <form onSubmit={async (e) => {
          e.preventDefault()
          try {
            const result = await verifyTotp(mfaCode)
            if (result.verified) {
              onSignIn?.()
              if (typeof window !== 'undefined') window.location.href = afterSignInUrl
            }
          } catch { /* error tracked by useMfa hook */ }
        }}>
          <p style={{ fontSize: 14, marginBottom: 12, color: '#6b7280' }}>
            {t('signIn.mfa.prompt')}
          </p>
          <input
            type="text" placeholder="000000" value={mfaCode}
            onChange={e => setMfaCode(e.target.value)}
            style={{ ...s.input, textAlign: 'center', letterSpacing: 8, fontSize: 20 }}
            maxLength={6} required
          />
          <button type="submit" disabled={mfaLoading} style={{ ...s.button, marginTop: 8, opacity: mfaLoading ? 0.7 : 1 }}>
            {mfaLoading ? t('signIn.mfa.loading') : t('signIn.mfa.button')}
          </button>
        </form>
      )}
    </div>
  )
}

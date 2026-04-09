import React from 'react'
import { NucleusProvider } from '@cntm-labs/react/provider'
import { en } from '@cntm-labs/react/i18n/locales/en'
import { SignIn } from '@cntm-labs/react/components/sign-in'
import { SignUp } from '@cntm-labs/react/components/sign-up'
import type { OAuthProvider } from '@cntm-labs/react/client/types'

const allProviders: OAuthProvider[] = [
  'google', 'github', 'apple', 'microsoft',
  'facebook', 'discord', 'twitter', 'linkedin', 'slack',
]

export function OAuthSignIn() {
  return (
    <NucleusProvider publishableKey="pk_test_e2e" locale={en}>
      <div data-testid="oauth-sign-in" style={{ padding: 24, maxWidth: 400, margin: '0 auto' }}>
        <SignIn oauthProviders={allProviders} />
      </div>
    </NucleusProvider>
  )
}

export function OAuthSignUp() {
  return (
    <NucleusProvider publishableKey="pk_test_e2e" locale={en}>
      <div data-testid="oauth-sign-up" style={{ padding: 24, maxWidth: 400, margin: '0 auto' }}>
        <SignUp oauthProviders={allProviders} />
      </div>
    </NucleusProvider>
  )
}

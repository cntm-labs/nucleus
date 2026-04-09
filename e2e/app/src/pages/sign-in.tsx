import React from 'react'
import { NucleusProvider } from '@cntm-labs/react/provider'
import { I18nContext } from '@cntm-labs/react/i18n'
import { en } from '@cntm-labs/react/i18n/locales/en'
import { th } from '@cntm-labs/react/i18n/locales/th'
import { SignIn } from '@cntm-labs/react/components/sign-in'

export function SignInEnglish() {
  return (
    <NucleusProvider publishableKey="pk_test_e2e" locale={en}>
      <div data-testid="sign-in-en" style={{ padding: 24, maxWidth: 400, margin: '0 auto' }}>
        <SignIn />
      </div>
    </NucleusProvider>
  )
}

export function SignInThai() {
  return (
    <NucleusProvider publishableKey="pk_test_e2e" locale={th}>
      <div data-testid="sign-in-th" style={{ padding: 24, maxWidth: 400, margin: '0 auto' }}>
        <SignIn />
      </div>
    </NucleusProvider>
  )
}

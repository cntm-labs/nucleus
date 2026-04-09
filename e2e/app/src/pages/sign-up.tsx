import React from 'react'
import { NucleusProvider } from '@cntm-labs/react/provider'
import { en } from '@cntm-labs/react/i18n/locales/en'
import { th } from '@cntm-labs/react/i18n/locales/th'
import { SignUp } from '@cntm-labs/react/components/sign-up'

export function SignUpEnglish() {
  return (
    <NucleusProvider publishableKey="pk_test_e2e" locale={en}>
      <div data-testid="sign-up-en" style={{ padding: 24, maxWidth: 400, margin: '0 auto' }}>
        <SignUp />
      </div>
    </NucleusProvider>
  )
}

export function SignUpThai() {
  return (
    <NucleusProvider publishableKey="pk_test_e2e" locale={th}>
      <div data-testid="sign-up-th" style={{ padding: 24, maxWidth: 400, margin: '0 auto' }}>
        <SignUp />
      </div>
    </NucleusProvider>
  )
}

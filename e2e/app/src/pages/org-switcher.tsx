import React from 'react'
import { NucleusProvider } from '@cntm-labs/react/provider'
import { en } from '@cntm-labs/react/i18n/locales/en'
import { th } from '@cntm-labs/react/i18n/locales/th'
import { OrgSwitcher } from '@cntm-labs/react/components/org-switcher'

export function OrgSwitcherEnglish() {
  return (
    <NucleusProvider publishableKey="pk_test_e2e" locale={en}>
      <div data-testid="org-switcher-en" style={{ padding: 24 }}>
        <OrgSwitcher />
      </div>
    </NucleusProvider>
  )
}

export function OrgSwitcherThai() {
  return (
    <NucleusProvider publishableKey="pk_test_e2e" locale={th}>
      <div data-testid="org-switcher-th" style={{ padding: 24 }}>
        <OrgSwitcher />
      </div>
    </NucleusProvider>
  )
}

import React from 'react'
import { NucleusProvider } from '@cntm-labs/react/provider'
import { th } from '@cntm-labs/react/i18n/locales/th'
import { OrgProfile } from '@cntm-labs/react/components/org-profile'

export function OrgProfileThai() {
  return (
    <NucleusProvider publishableKey="pk_test_e2e" locale={th}>
      <div data-testid="org-profile-th" style={{ padding: 24, maxWidth: 400, margin: '0 auto' }}>
        <OrgProfile />
      </div>
    </NucleusProvider>
  )
}

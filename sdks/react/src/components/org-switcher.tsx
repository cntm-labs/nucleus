import React, { useState } from 'react'
import { useNucleus } from '../provider'
import type { NucleusOrganization } from '../client/types'

export interface OrgSwitcherProps {
  appearance?: Record<string, unknown>
}

export function OrgSwitcher(_props: OrgSwitcherProps) {
  const { organization, _setOrganization } = useNucleus()
  const [isOpen, setIsOpen] = useState(false)
  const [organizations] = useState<NucleusOrganization[]>([])

  return (
    <div style={{ position: 'relative', display: 'inline-block' }}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        style={{
          padding: '8px 16px', border: '1px solid #ddd', borderRadius: 4,
          background: 'white', cursor: 'pointer', fontSize: 14,
        }}
      >
        {organization?.name ?? 'Select Organization'}
      </button>
      {isOpen && (
        <div style={{
          position: 'absolute', top: 40, left: 0, background: 'white',
          border: '1px solid #ddd', borderRadius: 8, padding: 4, minWidth: 200,
          boxShadow: '0 4px 12px rgba(0,0,0,0.1)',
        }}>
          {organizations.length === 0 ? (
            <div style={{ padding: '8px 12px', color: '#999', fontSize: 14 }}>No organizations</div>
          ) : (
            organizations.map(org => (
              <button
                key={org.id}
                onClick={() => { _setOrganization(org); setIsOpen(false) }}
                style={{
                  width: '100%', padding: '8px 12px', border: 'none', background: 'none',
                  textAlign: 'left', cursor: 'pointer', borderRadius: 4, fontSize: 14,
                }}
              >
                {org.name}
              </button>
            ))
          )}
        </div>
      )}
    </div>
  )
}

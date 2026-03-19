'use client'
import React from 'react'
import { useNucleus } from '../provider'

export interface OrgProfileProps {
  appearance?: Record<string, unknown>
}

export function OrgProfile(_props: OrgProfileProps) {
  const { organization, isLoaded } = useNucleus()

  if (!isLoaded) {
    return <div style={{ padding: 24 }}>Loading...</div>
  }

  if (!organization) {
    return <div style={{ padding: 24 }}>No organization selected</div>
  }

  return (
    <div style={{ maxWidth: 600, margin: '0 auto', padding: 24 }}>
      <h2 style={{ fontSize: 24, fontWeight: 'bold', marginBottom: 16 }}>Organization</h2>
      <div style={{ border: '1px solid #ddd', borderRadius: 8, padding: 16 }}>
        <div style={{ marginBottom: 8 }}>
          <strong>Name:</strong> {organization.name}
        </div>
        <div>
          <strong>Slug:</strong> {organization.slug}
        </div>
      </div>
    </div>
  )
}

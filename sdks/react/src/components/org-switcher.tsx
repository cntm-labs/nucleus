import React, { useState, useEffect } from 'react'
import { useOrganizationList } from '../hooks/use-organization-list'
import { useNucleus } from '../provider'
import { useStyles } from './appearance'
import { useTranslation } from '../i18n'

export interface OrgSwitcherProps {
  afterCreateUrl?: string
}

export function OrgSwitcher({ afterCreateUrl }: OrgSwitcherProps) {
  const { organization } = useNucleus()
  const { organizations, fetchOrganizations, createOrganization, setActiveOrganization, isLoading } = useOrganizationList()
  const [isOpen, setIsOpen] = useState(false)
  const [showCreate, setShowCreate] = useState(false)
  const [newName, setNewName] = useState('')
  const [newSlug, setNewSlug] = useState('')
  const s = useStyles()
  const t = useTranslation()

  useEffect(() => {
    if (isOpen) fetchOrganizations().catch(() => {})
  }, [isOpen, fetchOrganizations])

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      const org = await createOrganization(newName, newSlug)
      setActiveOrganization(org)
      setShowCreate(false)
      setNewName('')
      setNewSlug('')
      setIsOpen(false)
      if (afterCreateUrl && typeof window !== 'undefined') window.location.href = afterCreateUrl
    } catch { /* error tracked */ }
  }

  return (
    <div style={{ position: 'relative', display: 'inline-block' }}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        style={{
          padding: '8px 16px', border: '1px solid var(--nucleus-border, #d1d5db)',
          borderRadius: 'var(--nucleus-radius, 6px)',
          background: 'white', cursor: 'pointer', fontSize: 14, fontWeight: 500,
        }}
      >
        {organization?.name ?? t('orgSwitcher.select')}
      </button>
      {isOpen && (
        <div style={{
          position: 'absolute', top: 44, left: 0, background: 'white',
          border: '1px solid #e5e7eb', borderRadius: 8, padding: 4, minWidth: 220,
          boxShadow: '0 4px 12px rgba(0,0,0,0.1)', zIndex: 50,
        }}>
          {isLoading && <div style={{ padding: '8px 12px', color: '#9ca3af', fontSize: 13 }}>{t('orgSwitcher.loading')}</div>}

          {!isLoading && organizations.length === 0 && !showCreate && (
            <div style={{ padding: '8px 12px', color: '#9ca3af', fontSize: 14 }}>{t('orgSwitcher.empty')}</div>
          )}

          {!showCreate && organizations.map(org => (
            <button
              key={org.id}
              onClick={() => { setActiveOrganization(org); setIsOpen(false) }}
              style={{
                width: '100%', padding: '8px 12px', border: 'none',
                background: organization?.id === org.id ? '#f3f4f6' : 'none',
                textAlign: 'left', cursor: 'pointer', borderRadius: 4, fontSize: 14,
              }}
            >
              {org.name}
            </button>
          ))}

          {!showCreate && (
            <>
              <div style={{ height: 1, background: '#e5e7eb', margin: '4px 0' }} />
              <button
                onClick={() => setShowCreate(true)}
                style={{
                  width: '100%', padding: '8px 12px', border: 'none', background: 'none',
                  textAlign: 'left', cursor: 'pointer', borderRadius: 4, fontSize: 14,
                  color: 'var(--nucleus-primary, #4c6ef5)',
                }}
              >
                {t('orgSwitcher.create')}
              </button>
            </>
          )}

          {showCreate && (
            <form onSubmit={handleCreate} style={{ padding: 8 }}>
              <input
                type="text" placeholder={t('orgSwitcher.name')} value={newName}
                onChange={e => { setNewName(e.target.value); setNewSlug(e.target.value.toLowerCase().replace(/[^a-z0-9]+/g, '-')) }}
                style={{ ...s.input, fontSize: 13 }} required
              />
              <input
                type="text" placeholder={t('orgSwitcher.slug')} value={newSlug}
                onChange={e => setNewSlug(e.target.value)}
                style={{ ...s.input, fontSize: 13, marginBottom: 8 }} required
              />
              <div style={{ display: 'flex', gap: 4 }}>
                <button type="button" onClick={() => setShowCreate(false)} style={{ ...s.secondaryButton, flex: 1, fontSize: 13 }}>{t('orgSwitcher.cancel')}</button>
                <button type="submit" style={{ ...s.button, flex: 1, fontSize: 13 }}>{t('orgSwitcher.createButton')}</button>
              </div>
            </form>
          )}
        </div>
      )}
    </div>
  )
}

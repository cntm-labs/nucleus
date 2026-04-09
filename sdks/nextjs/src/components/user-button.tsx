'use client'
import React, { useState } from 'react'
import { useNucleus } from '../provider'
import { useTranslation } from '../i18n'

export interface UserButtonProps {
  afterSignOutUrl?: string
}

export function UserButton({ afterSignOutUrl = '/' }: UserButtonProps) {
  const { user, isSignedIn, signOut } = useNucleus()
  const [menuOpen, setMenuOpen] = useState(false)
  const t = useTranslation()

  if (!isSignedIn || !user) return null

  const initials = [user.first_name, user.last_name]
    .filter(Boolean).map(n => n![0]).join('').toUpperCase()
    || user.email[0].toUpperCase()

  return (
    <div style={{ position: 'relative', display: 'inline-block' }}>
      <button onClick={() => setMenuOpen(!menuOpen)} style={{
        width: 36, height: 36, borderRadius: '50%', border: 'none',
        background: 'var(--nucleus-primary, #4c6ef5)', color: 'white',
        cursor: 'pointer', fontSize: 14, fontWeight: 'bold', overflow: 'hidden',
      }}>
        {user.avatar_url ? <img src={user.avatar_url} alt="" style={{ width: 36, height: 36, borderRadius: '50%' }} /> : initials}
      </button>
      {menuOpen && (
        <div style={{
          position: 'absolute', top: 44, right: 0, background: 'white',
          border: '1px solid #e5e7eb', borderRadius: 8, padding: 8, minWidth: 200,
          boxShadow: '0 4px 12px rgba(0,0,0,0.1)', zIndex: 50,
        }}>
          <div style={{ padding: '8px 12px', borderBottom: '1px solid #f3f4f6', marginBottom: 4 }}>
            <div style={{ fontWeight: 600, fontSize: 14 }}>{user.first_name} {user.last_name}</div>
            <div style={{ fontSize: 12, color: '#6b7280' }}>{user.email}</div>
          </div>
          <button onClick={async () => {
            await signOut(); setMenuOpen(false)
            if (typeof window !== 'undefined') window.location.href = afterSignOutUrl
          }} style={{
            width: '100%', padding: '8px 12px', border: 'none', background: 'none',
            textAlign: 'left', cursor: 'pointer', borderRadius: 4, fontSize: 14, color: '#dc2626',
          }}>{t('userButton.signOut')}</button>
        </div>
      )}
    </div>
  )
}

import React, { useState } from 'react'
import { useNucleus } from '../provider'
import { useProfile } from '../hooks/use-profile'
import { useStyles } from './appearance'
import { useTranslation } from '../i18n'

export interface UserProfileProps {
  onUpdate?: () => void
}

export function UserProfile({ onUpdate }: UserProfileProps) {
  const { user } = useNucleus()
  const { updateProfile, updatePassword, isLoading, error } = useProfile()
  const [firstName, setFirstName] = useState(user?.first_name ?? '')
  const [lastName, setLastName] = useState(user?.last_name ?? '')
  const [currentPassword, setCurrentPassword] = useState('')
  const [newPassword, setNewPassword] = useState('')
  const [section, setSection] = useState<'profile' | 'password'>('profile')
  const [success, setSuccess] = useState<string | null>(null)
  const s = useStyles()
  const t = useTranslation()

  if (!user) return null

  const handleProfileUpdate = async (e: React.FormEvent) => {
    e.preventDefault()
    setSuccess(null)
    try {
      await updateProfile({ first_name: firstName, last_name: lastName })
      setSuccess('Profile updated')
      onUpdate?.()
    } catch { /* error tracked by hook */ }
  }

  const handlePasswordUpdate = async (e: React.FormEvent) => {
    e.preventDefault()
    setSuccess(null)
    try {
      await updatePassword(currentPassword, newPassword)
      setSuccess('Password updated')
      setCurrentPassword('')
      setNewPassword('')
    } catch { /* error tracked by hook */ }
  }

  return (
    <div style={s.card}>
      <h2 style={s.title}>{t('userProfile.title')}</h2>
      {error && <div style={s.error}>{error}</div>}
      {success && <div style={{ ...s.error, background: '#f0fdf4', color: '#16a34a' }}>{success}</div>}

      <div style={{ display: 'flex', gap: 8, marginBottom: 16 }}>
        <button onClick={() => setSection('profile')} style={{ ...s.secondaryButton, flex: 1, background: section === 'profile' ? '#e5e7eb' : undefined }}>
          {t('userProfile.profileTab')}
        </button>
        <button onClick={() => setSection('password')} style={{ ...s.secondaryButton, flex: 1, background: section === 'password' ? '#e5e7eb' : undefined }}>
          {t('userProfile.passwordTab')}
        </button>
      </div>

      {section === 'profile' && (
        <form onSubmit={handleProfileUpdate}>
          <label style={{ fontSize: 13, color: '#6b7280', marginBottom: 4, display: 'block' }}>{t('userProfile.email')}</label>
          <input type="email" value={user.email} disabled style={{ ...s.input, opacity: 0.6 }} />
          <label style={{ fontSize: 13, color: '#6b7280', marginBottom: 4, display: 'block' }}>{t('userProfile.firstName')}</label>
          <input type="text" value={firstName} onChange={e => setFirstName(e.target.value)} style={s.input} />
          <label style={{ fontSize: 13, color: '#6b7280', marginBottom: 4, display: 'block' }}>{t('userProfile.lastName')}</label>
          <input type="text" value={lastName} onChange={e => setLastName(e.target.value)} style={{ ...s.input, marginBottom: 16 }} />
          <button type="submit" disabled={isLoading} style={{ ...s.button, opacity: isLoading ? 0.7 : 1 }}>
            {isLoading ? t('userProfile.saving') : t('userProfile.save')}
          </button>
        </form>
      )}

      {section === 'password' && (
        <form onSubmit={handlePasswordUpdate}>
          <input
            type="password" placeholder={t('userProfile.currentPassword')} value={currentPassword}
            onChange={e => setCurrentPassword(e.target.value)} style={s.input} required
          />
          <input
            type="password" placeholder={t('userProfile.newPassword')} value={newPassword}
            onChange={e => setNewPassword(e.target.value)} style={{ ...s.input, marginBottom: 16 }} required
          />
          <button type="submit" disabled={isLoading} style={{ ...s.button, opacity: isLoading ? 0.7 : 1 }}>
            {isLoading ? t('userProfile.updating') : t('userProfile.updatePassword')}
          </button>
        </form>
      )}
    </div>
  )
}

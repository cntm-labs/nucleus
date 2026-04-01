'use client'
import React, { useState, useEffect, useCallback } from 'react'
import { useNucleus } from '../provider'
import { useStyles } from './appearance'
import { useTranslation } from '../i18n'
import type { NucleusMember } from '../client/types'

export interface OrgProfileProps {
  onInvite?: () => void
}

export function OrgProfile({ onInvite }: OrgProfileProps) {
  const { organization, _api, getToken } = useNucleus()
  const [members, setMembers] = useState<NucleusMember[]>([])
  const [inviteEmail, setInviteEmail] = useState('')
  const [inviteRole, setInviteRole] = useState('member')
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const s = useStyles()
  const t = useTranslation()

  const loadMembers = useCallback(async () => {
    if (!organization) return
    try { setMembers(await _api.getMembers(getToken()!, organization.id)) } catch { /* ignore */ }
  }, [_api, getToken, organization])

  useEffect(() => { loadMembers() }, [loadMembers])

  if (!organization) return <div style={s.card}><p style={{ color: '#6b7280' }}>{t('orgProfile.noOrg')}</p></div>

  const handleInvite = async (e: React.FormEvent) => {
    e.preventDefault(); setIsLoading(true); setError(null)
    try { await _api.createInvitation(getToken()!, organization.id, inviteEmail, inviteRole); setInviteEmail(''); onInvite?.() }
    catch (err) { setError(err instanceof Error ? err.message : 'Invite failed') }
    finally { setIsLoading(false) }
  }

  return (
    <div style={s.card}>
      <h2 style={s.title}>{organization.name}</h2>
      {error && <div style={s.error}>{error}</div>}
      <div style={{ marginBottom: 16 }}>
        <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 8 }}>{t('orgProfile.members')}</h3>
        {members.map(member => (
          <div key={member.id} style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '8px 0', borderBottom: '1px solid #f3f4f6' }}>
            <div>
              <div style={{ fontSize: 14, fontWeight: 500 }}>{member.first_name} {member.last_name}</div>
              <div style={{ fontSize: 12, color: '#6b7280' }}>{member.email} &middot; {member.role}</div>
            </div>
            <button onClick={async () => {
              try { await _api.removeMember(getToken()!, organization.id, member.id); setMembers(prev => prev.filter(m => m.id !== member.id)) }
              catch (err) { setError(err instanceof Error ? err.message : 'Remove failed') }
            }} style={{ border: 'none', background: 'none', color: '#dc2626', cursor: 'pointer', fontSize: 13 }}>{t('orgProfile.remove')}</button>
          </div>
        ))}
      </div>
      <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 8 }}>{t('orgProfile.invite')}</h3>
      <form onSubmit={handleInvite}>
        <input type="email" placeholder={t('orgProfile.email')} value={inviteEmail} onChange={e => setInviteEmail(e.target.value)} style={s.input} required />
        <select value={inviteRole} onChange={e => setInviteRole(e.target.value)} style={{ ...s.input, marginBottom: 16 }}>
          <option value="member">{t('orgProfile.roleMember')}</option>
          <option value="admin">{t('orgProfile.roleAdmin')}</option>
        </select>
        <button type="submit" disabled={isLoading} style={{ ...s.button, opacity: isLoading ? 0.7 : 1 }}>{isLoading ? t('orgProfile.sending') : t('orgProfile.sendInvite')}</button>
      </form>
    </div>
  )
}

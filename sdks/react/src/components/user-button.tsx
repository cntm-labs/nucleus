import React, { useState, useEffect } from 'react'
import { useNucleus } from '../provider'
import { useSessionList } from '../hooks/use-session-list'
import { useStyles } from './appearance'

export interface UserButtonProps {
  afterSignOutUrl?: string
  showSessions?: boolean
}

export function UserButton({ afterSignOutUrl = '/', showSessions = false }: UserButtonProps) {
  const { user, isSignedIn, signOut } = useNucleus()
  const { sessions, fetchSessions, switchSession } = useSessionList()
  const [menuOpen, setMenuOpen] = useState(false)
  const s = useStyles()

  useEffect(() => {
    if (menuOpen && showSessions) {
      fetchSessions().catch(() => {})
    }
  }, [menuOpen, showSessions, fetchSessions])

  if (!isSignedIn || !user) return null

  const initials = [user.first_name, user.last_name]
    .filter(Boolean).map(n => n![0]).join('').toUpperCase()
    || user.email[0].toUpperCase()

  return (
    <div style={{ position: 'relative', display: 'inline-block' }}>
      <button
        onClick={() => setMenuOpen(!menuOpen)}
        style={{
          width: 36, height: 36, borderRadius: '50%', border: 'none',
          background: 'var(--nucleus-primary, #4c6ef5)', color: 'white',
          cursor: 'pointer', fontSize: 14, fontWeight: 'bold', overflow: 'hidden',
        }}
      >
        {user.avatar_url
          ? <img src={user.avatar_url} alt="" style={{ width: 36, height: 36, borderRadius: '50%' }} />
          : initials}
      </button>
      {menuOpen && (
        <div style={{
          position: 'absolute', top: 44, right: 0, background: 'white',
          border: '1px solid #e5e7eb', borderRadius: 8, padding: 8, minWidth: 220,
          boxShadow: '0 4px 12px rgba(0,0,0,0.1)', zIndex: 50,
        }}>
          <div style={{ padding: '8px 12px', borderBottom: '1px solid #f3f4f6', marginBottom: 4 }}>
            <div style={{ fontWeight: 600, fontSize: 14 }}>{user.first_name} {user.last_name}</div>
            <div style={{ fontSize: 12, color: '#6b7280' }}>{user.email}</div>
          </div>

          {showSessions && sessions.length > 1 && (
            <div style={{ borderBottom: '1px solid #f3f4f6', marginBottom: 4, paddingBottom: 4 }}>
              <div style={{ fontSize: 11, color: '#9ca3af', padding: '4px 12px', textTransform: 'uppercase' }}>Sessions</div>
              {sessions.map(sess => (
                <button
                  key={sess.id}
                  onClick={async () => { await switchSession(sess.id); setMenuOpen(false) }}
                  style={{
                    width: '100%', padding: '6px 12px', border: 'none', background: 'none',
                    textAlign: 'left', cursor: 'pointer', borderRadius: 4, fontSize: 13,
                  }}
                >
                  Session {sess.id.slice(0, 8)}...
                </button>
              ))}
            </div>
          )}

          <button
            onClick={async () => {
              await signOut()
              setMenuOpen(false)
              if (typeof window !== 'undefined') window.location.href = afterSignOutUrl
            }}
            style={{
              width: '100%', padding: '8px 12px', border: 'none', background: 'none',
              textAlign: 'left', cursor: 'pointer', borderRadius: 4, fontSize: 14,
              color: '#dc2626',
            }}
          >
            Sign Out
          </button>
        </div>
      )}
    </div>
  )
}

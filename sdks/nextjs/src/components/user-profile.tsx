'use client'
import React from 'react'
import { useNucleus } from '../provider'

export interface UserProfileProps {
  appearance?: Record<string, unknown>
}

export function UserProfile(_props: UserProfileProps) {
  const { user, isLoaded, isSignedIn } = useNucleus()

  if (!isLoaded) {
    return <div style={{ padding: 24 }}>Loading...</div>
  }

  if (!isSignedIn || !user) {
    return <div style={{ padding: 24 }}>Not signed in</div>
  }

  return (
    <div style={{ maxWidth: 600, margin: '0 auto', padding: 24 }}>
      <h2 style={{ fontSize: 24, fontWeight: 'bold', marginBottom: 16 }}>Profile</h2>
      <div style={{ border: '1px solid #ddd', borderRadius: 8, padding: 16 }}>
        {user.avatar_url && (
          <img src={user.avatar_url} alt="" style={{ width: 64, height: 64, borderRadius: '50%', marginBottom: 16 }} />
        )}
        <div style={{ marginBottom: 8 }}>
          <strong>Name:</strong> {user.first_name} {user.last_name}
        </div>
        <div style={{ marginBottom: 8 }}>
          <strong>Email:</strong> {user.email}
          {user.email_verified && <span style={{ color: 'green', marginLeft: 8 }}>(verified)</span>}
        </div>
        <div>
          <strong>Member since:</strong> {new Date(user.created_at).toLocaleDateString()}
        </div>
      </div>
    </div>
  )
}

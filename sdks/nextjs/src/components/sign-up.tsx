'use client'
import React, { useState } from 'react'

export interface SignUpProps {
  afterSignUpUrl?: string
  routing?: 'hash' | 'path'
}

export function SignUp({ afterSignUpUrl = '/' }: SignUpProps) {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [firstName, setFirstName] = useState('')
  const [lastName, setLastName] = useState('')

  void afterSignUpUrl

  return (
    <div style={{ maxWidth: 400, margin: '0 auto', padding: 24 }}>
      <h2 style={{ fontSize: 24, fontWeight: 'bold', marginBottom: 16 }}>Sign Up</h2>
      <form onSubmit={e => { e.preventDefault(); /* TODO */ }}>
        <input
          type="text"
          placeholder="First Name"
          value={firstName}
          onChange={e => setFirstName(e.target.value)}
          style={{ width: '100%', padding: 8, marginBottom: 8, border: '1px solid #ddd', borderRadius: 4 }}
        />
        <input
          type="text"
          placeholder="Last Name"
          value={lastName}
          onChange={e => setLastName(e.target.value)}
          style={{ width: '100%', padding: 8, marginBottom: 8, border: '1px solid #ddd', borderRadius: 4 }}
        />
        <input
          type="email"
          placeholder="Email"
          value={email}
          onChange={e => setEmail(e.target.value)}
          style={{ width: '100%', padding: 8, marginBottom: 8, border: '1px solid #ddd', borderRadius: 4 }}
        />
        <input
          type="password"
          placeholder="Password"
          value={password}
          onChange={e => setPassword(e.target.value)}
          style={{ width: '100%', padding: 8, marginBottom: 16, border: '1px solid #ddd', borderRadius: 4 }}
        />
        <button
          type="submit"
          style={{ width: '100%', padding: 10, background: '#4c6ef5', color: 'white', border: 'none', borderRadius: 4, cursor: 'pointer' }}
        >
          Sign Up
        </button>
      </form>
    </div>
  )
}

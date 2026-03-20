import React, { useState } from 'react'
import { useSignUp } from '../hooks/use-sign-up'

export interface SignUpProps {
  afterSignUpUrl?: string
  routing?: 'hash' | 'path'
  onSignUp?: () => void
}

export function SignUp({ afterSignUpUrl = '/', onSignUp }: SignUpProps) {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [firstName, setFirstName] = useState('')
  const [lastName, setLastName] = useState('')
  const { signUp, isLoading, error } = useSignUp()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await signUp(email, password, firstName, lastName)
      onSignUp?.()
      if (typeof window !== 'undefined') {
        window.location.href = afterSignUpUrl
      }
    } catch {
      // error is tracked by useSignUp
    }
  }

  return (
    <div style={{ maxWidth: 400, margin: '0 auto', padding: 24 }}>
      <h2 style={{ fontSize: 24, fontWeight: 'bold', marginBottom: 16 }}>Sign Up</h2>
      {error && (
        <div style={{ padding: 8, marginBottom: 12, background: '#fee', color: '#c00', borderRadius: 4, fontSize: 14 }}>
          {error}
        </div>
      )}
      <form onSubmit={handleSubmit}>
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
          disabled={isLoading}
          style={{ width: '100%', padding: 10, background: '#4c6ef5', color: 'white', border: 'none', borderRadius: 4, cursor: 'pointer', opacity: isLoading ? 0.7 : 1 }}
        >
          {isLoading ? 'Creating account...' : 'Sign Up'}
        </button>
      </form>
    </div>
  )
}

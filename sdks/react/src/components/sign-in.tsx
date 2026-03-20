import React, { useState } from 'react'
import { useSignIn } from '../hooks/use-sign-in'

export interface SignInProps {
  afterSignInUrl?: string
  routing?: 'hash' | 'path'
  onSignIn?: () => void
}

export function SignIn({ afterSignInUrl = '/', onSignIn }: SignInProps) {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const { signIn, isLoading, error } = useSignIn()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await signIn(email, password)
      onSignIn?.()
      if (typeof window !== 'undefined') {
        window.location.href = afterSignInUrl
      }
    } catch {
      // error is tracked by useSignIn
    }
  }

  return (
    <div style={{ maxWidth: 400, margin: '0 auto', padding: 24 }}>
      <h2 style={{ fontSize: 24, fontWeight: 'bold', marginBottom: 16 }}>Sign In</h2>
      {error && (
        <div style={{ padding: 8, marginBottom: 12, background: '#fee', color: '#c00', borderRadius: 4, fontSize: 14 }}>
          {error}
        </div>
      )}
      <form onSubmit={handleSubmit}>
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
          {isLoading ? 'Signing in...' : 'Sign In'}
        </button>
      </form>
    </div>
  )
}

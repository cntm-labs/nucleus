import { createContext, useContext, useState, ReactNode } from 'react'
import { api } from './api'

interface AuthState {
  isAuthenticated: boolean
  account: { id: string; email: string; name: string } | null
  login: (email: string, password: string) => Promise<void>
  logout: () => void
}

const AuthContext = createContext<AuthState | null>(null)

export function AuthProvider({ children }: { children: ReactNode }) {
  const [account, setAccount] = useState<AuthState['account']>(null)

  const login = async (email: string, _password: string) => {
    // TODO: call actual login endpoint
    setAccount({ id: '1', email, name: email.split('@')[0] })
    api.setToken('placeholder-token')
  }

  const logout = () => {
    setAccount(null)
    api.setToken(null)
  }

  return (
    <AuthContext.Provider value={{ isAuthenticated: !!account, account, login, logout }}>
      {children}
    </AuthContext.Provider>
  )
}

export function useAuth() {
  const ctx = useContext(AuthContext)
  if (!ctx) throw new Error('useAuth must be used within AuthProvider')
  return ctx
}

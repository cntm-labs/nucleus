import { createContext, useContext, useEffect, useState, ReactNode } from 'react'
import { api, accountApi, Account } from './api'

interface AuthState {
  isAuthenticated: boolean
  loading: boolean
  account: Account | null
  login: (email: string, password: string) => Promise<void>
  logout: () => Promise<void>
}

const AuthContext = createContext<AuthState | null>(null)

export function AuthProvider({ children }: { children: ReactNode }) {
  const [account, setAccount] = useState<Account | null>(null)
  const [loading, setLoading] = useState(true)

  // On mount, if we have a stored token, try to fetch the current account.
  useEffect(() => {
    let cancelled = false
    accountApi
      .me()
      .then((acc) => {
        if (!cancelled) setAccount(acc)
      })
      .catch(() => {
        // Token invalid/expired — clear it
        api.setToken(null)
        if (!cancelled) setAccount(null)
      })
      .finally(() => {
        if (!cancelled) setLoading(false)
      })
    return () => {
      cancelled = true
    }
  }, [])

  const login = async (email: string, password: string) => {
    const { account: acc, token } = await accountApi.signIn(email, password)
    api.setToken(token)
    setAccount(acc)
  }

  const logout = async () => {
    try {
      await accountApi.signOut()
    } catch {
      // best-effort; clear local state regardless
    } finally {
      api.setToken(null)
      setAccount(null)
    }
  }

  return (
    <AuthContext.Provider
      value={{ isAuthenticated: !!account, loading, account, login, logout }}
    >
      {children}
    </AuthContext.Provider>
  )
}

export function useAuth() {
  const ctx = useContext(AuthContext)
  if (!ctx) throw new Error('useAuth must be used within AuthProvider')
  return ctx
}

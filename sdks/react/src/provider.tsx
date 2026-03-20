import React, { createContext, useContext, useState, useEffect, useCallback, ReactNode } from 'react'
import { NucleusUser, NucleusSession, NucleusOrganization } from './client/types'
import { NucleusApi } from './client/api'

interface NucleusContextValue {
  user: NucleusUser | null
  isLoaded: boolean
  isSignedIn: boolean
  session: NucleusSession | null
  organization: NucleusOrganization | null
  signOut: () => Promise<void>
  getToken: () => Promise<string | null>
  /** @internal */
  _api: NucleusApi
  /** @internal */
  _setUser: (user: NucleusUser | null) => void
  /** @internal */
  _setSession: (session: NucleusSession | null) => void
  /** @internal */
  _setOrganization: (org: NucleusOrganization | null) => void
}

const NucleusContext = createContext<NucleusContextValue | null>(null)

export interface NucleusProviderProps {
  publishableKey: string
  baseUrl?: string
  children: ReactNode
}

export function NucleusProvider({ publishableKey, baseUrl, children }: NucleusProviderProps) {
  const [user, setUser] = useState<NucleusUser | null>(null)
  const [session, setSession] = useState<NucleusSession | null>(null)
  const [organization, setOrganization] = useState<NucleusOrganization | null>(null)
  const [isLoaded, setIsLoaded] = useState(false)
  const [api] = useState(() => new NucleusApi({ publishableKey, baseUrl }))

  useEffect(() => {
    const storedToken = typeof window !== 'undefined'
      ? window.localStorage.getItem('__nucleus_session_token')
      : null

    if (storedToken) {
      api.getUser(storedToken)
        .then(u => {
          setUser(u)
          setSession({ id: '', token: storedToken, expires_at: '' })
        })
        .catch(() => {
          window.localStorage.removeItem('__nucleus_session_token')
        })
        .finally(() => setIsLoaded(true))
    } else {
      setIsLoaded(true)
    }
  }, [api])

  const signOut = useCallback(async () => {
    if (session?.token) {
      try { await api.signOut(session.token) } catch { /* best-effort */ }
    }
    setUser(null)
    setSession(null)
    setOrganization(null)
    if (typeof window !== 'undefined') {
      window.localStorage.removeItem('__nucleus_session_token')
    }
  }, [api, session])

  const getToken = useCallback(async () => session?.token || null, [session])

  return (
    <NucleusContext.Provider
      value={{
        user,
        isLoaded,
        isSignedIn: !!user,
        session,
        organization,
        signOut,
        getToken,
        _api: api,
        _setUser: setUser,
        _setSession: setSession,
        _setOrganization: setOrganization,
      }}
    >
      {children}
    </NucleusContext.Provider>
  )
}

export function useNucleus() {
  const ctx = useContext(NucleusContext)
  if (!ctx) throw new Error('useNucleus must be used within <NucleusProvider>')
  return ctx
}

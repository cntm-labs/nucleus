'use client'
import React, { createContext, useContext, useState, useEffect, useCallback, useRef, type ReactNode } from 'react'
import type { NucleusUser, NucleusSession, NucleusOrganization } from './client/types'
import { NucleusApi } from './client/api'
import { getSessionToken, setSessionToken, clearSessionToken, autoRefresh } from './client/session'

export interface NucleusContextValue {
  user: NucleusUser | null
  isLoaded: boolean
  isSignedIn: boolean
  session: NucleusSession | null
  organization: NucleusOrganization | null
  signOut: () => Promise<void>
  getToken: () => string | null
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

  const apiRef = useRef(new NucleusApi({ publishableKey, baseUrl }))

  useEffect(() => {
    const api = apiRef.current
    const token = getSessionToken()
    if (!token) { setIsLoaded(true); return }

    api.getUser(token)
      .then(u => {
        setUser(u)
        setSession({ id: '', token, refresh_token: '', expires_at: '', user_id: u.id })
      })
      .catch(() => clearSessionToken())
      .finally(() => setIsLoaded(true))
  }, [])

  // Auto-refresh timer
  useEffect(() => {
    if (!session?.expires_at) return
    const timer = setTimeout(async () => {
      const newToken = await autoRefresh(session.expires_at, async () => {
        const s = await apiRef.current.refreshSession(session.refresh_token)
        return { token: s.token, expires_at: s.expires_at }
      })
      if (newToken && newToken !== session.token) {
        setSession(prev => prev ? { ...prev, token: newToken } : null)
      }
    }, Math.max(0, new Date(session.expires_at).getTime() - 60_000 - Date.now()))
    return () => clearTimeout(timer)
  }, [session])

  const signOut = useCallback(async () => {
    if (session?.token) {
      try { await apiRef.current.signOut(session.token) } catch { /* best-effort */ }
    }
    setUser(null)
    setSession(null)
    setOrganization(null)
    clearSessionToken()
  }, [session])

  const getToken = useCallback(() => session?.token ?? null, [session])

  return (
    <NucleusContext.Provider
      value={{
        user, isLoaded, isSignedIn: !!user && !!session,
        session, organization, signOut, getToken,
        _api: apiRef.current,
        _setUser: setUser, _setSession: setSession, _setOrganization: setOrganization,
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

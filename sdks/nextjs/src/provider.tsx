'use client'
import React, { createContext, useContext, useState, useEffect, useCallback, useRef, type ReactNode } from 'react'
import type { NucleusUser, NucleusSession, NucleusOrganization } from './client/types'
import { NucleusApi } from './client/api'
import { getSessionToken, getRefreshToken, getExpiresAt, setSessionTokens, clearSessionTokens, autoRefresh } from './client/session'
import { I18nContext, type Locale } from './i18n'
import { en } from './i18n/locales/en'

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
  locale?: Locale
  children: ReactNode
}

export function NucleusProvider({ publishableKey, baseUrl, locale, children }: NucleusProviderProps) {
  const [user, setUser] = useState<NucleusUser | null>(null)
  const [session, setSession] = useState<NucleusSession | null>(null)
  const [organization, setOrganization] = useState<NucleusOrganization | null>(null)
  const [isLoaded, setIsLoaded] = useState(false)

  const apiRef = useRef(new NucleusApi({ publishableKey, baseUrl }))

  useEffect(() => {
    const api = apiRef.current
    const token = getSessionToken()
    const refreshToken = getRefreshToken()
    const expiresAt = getExpiresAt()
    if (!token) { setIsLoaded(true); return }

    // If we have refresh_token and expires_at, restore full session
    if (refreshToken && expiresAt) {
      const restoredSession: NucleusSession = {
        id: '', token, refresh_token: refreshToken, expires_at: expiresAt, user_id: '',
      }
      setSession(restoredSession)
      api.getUser(token)
        .then(u => { setUser(u); setSession(prev => prev ? { ...prev, user_id: u.id } : null) })
        .catch(() => clearSessionTokens())
        .finally(() => setIsLoaded(true))
    } else {
      // Incomplete session data — try to use token but can't auto-refresh
      api.getUser(token)
        .then(u => {
          setUser(u)
          setSession({ id: '', token, refresh_token: '', expires_at: '', user_id: u.id })
        })
        .catch(() => clearSessionTokens())
        .finally(() => setIsLoaded(true))
    }
  }, [])

  // Auto-refresh timer — only if we have valid expires_at
  useEffect(() => {
    if (!session?.expires_at || !session?.refresh_token) return
    const expiresMs = new Date(session.expires_at).getTime()
    if (isNaN(expiresMs)) return
    const refreshAt = Math.max(0, expiresMs - 60_000 - Date.now())
    const timer = setTimeout(async () => {
      const newToken = await autoRefresh(session.expires_at, async () => {
        const s = await apiRef.current.refreshSession(session.refresh_token)
        return { token: s.token, refresh_token: s.refresh_token, expires_at: s.expires_at }
      })
      if (newToken && newToken !== session.token) {
        const newExpiresAt = getExpiresAt()
        const newRefreshToken = getRefreshToken()
        setSession(prev => prev ? {
          ...prev, token: newToken,
          expires_at: newExpiresAt ?? prev.expires_at,
          refresh_token: newRefreshToken ?? prev.refresh_token,
        } : null)
      }
    }, refreshAt)
    return () => clearTimeout(timer)
  }, [session])

  const signOut = useCallback(async () => {
    if (session?.token) {
      try { await apiRef.current.signOut(session.token) } catch { /* best-effort */ }
    }
    setUser(null)
    setSession(null)
    setOrganization(null)
    clearSessionTokens()
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
      <I18nContext.Provider value={locale ?? en}>
        {children}
      </I18nContext.Provider>
    </NucleusContext.Provider>
  )
}

export function useNucleus() {
  const ctx = useContext(NucleusContext)
  if (!ctx) throw new Error('useNucleus must be used within <NucleusProvider>')
  return ctx
}

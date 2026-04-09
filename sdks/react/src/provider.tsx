import React, { createContext, useContext, useState, useEffect, useCallback, useRef, type ReactNode } from 'react'
import type { NucleusUser, NucleusSession, NucleusOrganization, AppearanceConfig } from './client/types'
import { NucleusApi } from './client/api'
import { SessionManager } from './client/session'
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
  _sessionManager: SessionManager
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
  appearance?: AppearanceConfig
  locale?: Locale
  children: ReactNode
}

export function NucleusProvider({ publishableKey, baseUrl, appearance, locale, children }: NucleusProviderProps) {
  const [user, setUser] = useState<NucleusUser | null>(null)
  const [session, setSession] = useState<NucleusSession | null>(null)
  const [organization, setOrganization] = useState<NucleusOrganization | null>(null)
  const [isLoaded, setIsLoaded] = useState(false)

  const apiRef = useRef<NucleusApi>(new NucleusApi({ publishableKey, baseUrl }))
  const sessionManagerRef = useRef<SessionManager>(
    new SessionManager(apiRef.current, (s) => setSession(s))
  )

  useEffect(() => {
    const sm = sessionManagerRef.current
    const api = apiRef.current
    const stored = sm.restore()

    if (stored) {
      const isExpired = new Date(stored.expires_at).getTime() <= Date.now()
      if (isExpired) {
        sm.refresh(stored.refresh_token)
          .then(async (s) => {
            if (s) {
              try { setUser(await api.getUser(s.token)) } catch { /* ignore */ }
            }
          })
          .finally(() => setIsLoaded(true))
      } else {
        const restoredSession: NucleusSession = {
          id: stored.session_id, token: stored.token,
          refresh_token: stored.refresh_token, expires_at: stored.expires_at, user_id: '',
        }
        setSession(restoredSession)
        sm.setSession(restoredSession)
        api.getUser(stored.token)
          .then(u => setUser(u))
          .catch(() => { sm.clearSession(); setUser(null) })
          .finally(() => setIsLoaded(true))
      }
    } else {
      setIsLoaded(true)
    }

    return () => sm.destroy()
  }, [])

  const signOut = useCallback(async () => {
    if (session?.token) {
      try { await apiRef.current.signOut(session.token) } catch { /* best-effort */ }
    }
    setUser(null)
    setOrganization(null)
    sessionManagerRef.current.clearSession()
  }, [session])

  const getToken = useCallback(() => session?.token ?? null, [session])

  const cssVars = appearance?.variables
    ? Object.entries(appearance.variables).reduce<Record<string, string>>((acc, [k, v]) => {
        acc[k.startsWith('--') ? k : `--nucleus-${k}`] = v
        return acc
      }, {})
    : undefined

  const content = cssVars ? <div style={cssVars as React.CSSProperties}>{children}</div> : children

  return (
    <NucleusContext.Provider
      value={{
        user, isLoaded, isSignedIn: !!user && !!session,
        session, organization, signOut, getToken,
        _api: apiRef.current,
        _sessionManager: sessionManagerRef.current,
        _setUser: setUser, _setSession: setSession, _setOrganization: setOrganization,
      }}
    >
      <I18nContext.Provider value={locale ?? en}>
        {content}
      </I18nContext.Provider>
    </NucleusContext.Provider>
  )
}

export function useNucleus() {
  const ctx = useContext(NucleusContext)
  if (!ctx) throw new Error('useNucleus must be used within <NucleusProvider>')
  return ctx
}

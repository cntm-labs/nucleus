'use client'
import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react'
import { NucleusUser, NucleusSession, NucleusOrganization } from './client/types'

interface NucleusContextValue {
  user: NucleusUser | null
  isLoaded: boolean
  isSignedIn: boolean
  session: NucleusSession | null
  organization: NucleusOrganization | null
  signOut: () => Promise<void>
  getToken: () => Promise<string | null>
}

const NucleusContext = createContext<NucleusContextValue | null>(null)

export function NucleusProvider({ publishableKey, children }: { publishableKey: string; children: ReactNode }) {
  const [user, setUser] = useState<NucleusUser | null>(null)
  const [session, setSession] = useState<NucleusSession | null>(null)
  const [organization, setOrganization] = useState<NucleusOrganization | null>(null)
  const [isLoaded, setIsLoaded] = useState(false)

  useEffect(() => {
    // TODO: Initialize session from cookies, load user
    void publishableKey
    setIsLoaded(true)
  }, [publishableKey])

  const signOut = async () => {
    setUser(null)
    setSession(null)
    setOrganization(null)
  }

  const getToken = async () => session?.token || null

  return (
    <NucleusContext.Provider value={{ user, isLoaded, isSignedIn: !!user, session, organization, signOut, getToken }}>
      {children}
    </NucleusContext.Provider>
  )
}

export function useNucleus() {
  const ctx = useContext(NucleusContext)
  if (!ctx) throw new Error('useNucleus must be used within NucleusProvider')
  return ctx
}

import { describe, it, expect, vi, beforeEach } from 'vitest'
import React from 'react'
import { renderHook, act } from '@testing-library/react'
import { NucleusProvider } from '../src/provider'
import { useAuth } from '../src/hooks/use-auth'
import { useUser } from '../src/hooks/use-user'
import { useSession } from '../src/hooks/use-session'
import { useOrganization } from '../src/hooks/use-organization'

const mockFetch = vi.fn()
vi.stubGlobal('fetch', mockFetch)

function wrapper({ children }: { children: React.ReactNode }) {
  return <NucleusProvider publishableKey="pk_test">{children}</NucleusProvider>
}

describe('hooks', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // Mock localStorage
    const store = new Map<string, string>()
    Object.defineProperty(window, 'localStorage', {
      value: {
        getItem: (key: string) => store.get(key) ?? null,
        setItem: (key: string, value: string) => store.set(key, value),
        removeItem: (key: string) => store.delete(key),
      },
      writable: true,
      configurable: true,
    })
  })

  describe('useAuth', () => {
    it('returns isLoaded true after mount', async () => {
      const { result } = renderHook(() => useAuth(), { wrapper })
      // isLoaded becomes true after useEffect
      await vi.waitFor(() => expect(result.current.isLoaded).toBe(true))
    })

    it('returns isSignedIn false initially', async () => {
      const { result } = renderHook(() => useAuth(), { wrapper })
      await vi.waitFor(() => expect(result.current.isLoaded).toBe(true))
      expect(result.current.isSignedIn).toBe(false)
    })

    it('returns userId null when not signed in', async () => {
      const { result } = renderHook(() => useAuth(), { wrapper })
      await vi.waitFor(() => expect(result.current.isLoaded).toBe(true))
      expect(result.current.userId).toBeNull()
    })
  })

  describe('useUser', () => {
    it('returns null user initially', async () => {
      const { result } = renderHook(() => useUser(), { wrapper })
      await vi.waitFor(() => expect(result.current.isLoaded).toBe(true))
      expect(result.current.user).toBeNull()
    })
  })

  describe('useSession', () => {
    it('returns null session initially', async () => {
      const { result } = renderHook(() => useSession(), { wrapper })
      await vi.waitFor(() => expect(result.current.isLoaded).toBe(true))
      expect(result.current.session).toBeNull()
      expect(result.current.isActive).toBe(false)
    })
  })

  describe('useOrganization', () => {
    it('returns null organization initially', async () => {
      const { result } = renderHook(() => useOrganization(), { wrapper })
      await vi.waitFor(() => expect(result.current.isLoaded).toBe(true))
      expect(result.current.organization).toBeNull()
      expect(result.current.isActive).toBe(false)
    })
  })
})

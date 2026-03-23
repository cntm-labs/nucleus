import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'

// Set up browser globals before importing modules that reference them
const sessionStore: Record<string, string> = {}
const listeners: Array<(e: any) => void> = []

const mockOpen = vi.fn(() => ({ closed: false }))

vi.stubGlobal('crypto', { randomUUID: () => 'test-state-uuid-1234' })
vi.stubGlobal('sessionStorage', {
  getItem: vi.fn((key: string) => sessionStore[key] ?? null),
  setItem: vi.fn((key: string, value: string) => { sessionStore[key] = value }),
  removeItem: vi.fn((key: string) => { delete sessionStore[key] }),
})
vi.stubGlobal('window', {
  location: { origin: 'https://app.test.com' },
  open: mockOpen,
  addEventListener: vi.fn((_type: string, fn: any) => { listeners.push(fn) }),
  removeEventListener: vi.fn((_type: string, fn: any) => {
    const idx = listeners.indexOf(fn)
    if (idx >= 0) listeners.splice(idx, 1)
  }),
})

import { NucleusApi } from '../src/api'
import { startOAuthFlow } from '../src/oauth'

describe('OAuth CSRF state parameter', () => {
  let api: NucleusApi

  beforeEach(() => {
    vi.clearAllMocks()
    Object.keys(sessionStore).forEach((k) => delete sessionStore[k])
    listeners.length = 0
    mockOpen.mockReturnValue({ closed: false })
    api = new NucleusApi('pk_test_123', 'https://api.test.com')
  })

  it('generates a random state and stores it in sessionStorage', () => {
    const promise = startOAuthFlow(api, 'google')

    expect(sessionStorage.setItem).toHaveBeenCalledWith(
      '__nucleus_oauth_state',
      'test-state-uuid-1234',
    )

    // Clean up: mark popup as closed
    mockOpen.mock.results[0].value.closed = true
    promise.catch(() => {})
  })

  it('includes state in OAuth URL', () => {
    const promise = startOAuthFlow(api, 'google')
    const openedUrl = mockOpen.mock.calls[0][0] as string
    expect(openedUrl).toContain('state=test-state-uuid-1234')

    mockOpen.mock.results[0].value.closed = true
    promise.catch(() => {})
  })

  it('rejects callback with mismatched state', async () => {
    vi.stubGlobal('fetch', vi.fn())

    const promise = startOAuthFlow(api, 'google')

    // Simulate callback with WRONG state
    for (const fn of [...listeners]) {
      fn({
        origin: 'https://app.test.com',
        data: { type: 'nucleus:oauth:callback', code: 'auth-code', state: 'wrong-state' },
      })
    }

    await expect(promise).rejects.toThrow('OAuth state mismatch')
    expect(fetch).not.toHaveBeenCalled()
  })

  it('accepts callback with matching state', async () => {
    const mockResult = { user: { id: '1' }, session: { token: 'tk' } }
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(
      new Response(JSON.stringify(mockResult), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    ))

    const promise = startOAuthFlow(api, 'google')

    // Simulate callback with CORRECT state
    for (const fn of [...listeners]) {
      fn({
        origin: 'https://app.test.com',
        data: { type: 'nucleus:oauth:callback', code: 'auth-code', state: 'test-state-uuid-1234' },
      })
    }

    const result = await promise
    expect(result).toEqual(mockResult)
    expect(sessionStorage.removeItem).toHaveBeenCalledWith('__nucleus_oauth_state')
  })
})

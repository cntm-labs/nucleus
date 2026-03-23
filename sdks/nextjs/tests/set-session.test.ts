import { describe, it, expect, vi, beforeEach } from 'vitest'

const { mockSet, mockDelete, mockCookies } = vi.hoisted(() => {
  const mockSet = vi.fn()
  const mockDelete = vi.fn()
  const mockCookies = vi.fn().mockResolvedValue({ set: mockSet, delete: mockDelete })
  return { mockSet, mockDelete, mockCookies }
})

vi.mock('next/headers', () => ({ cookies: mockCookies }))

import { setNucleusSession, clearNucleusSession } from '../src/server/set-session'

describe('setNucleusSession', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockCookies.mockResolvedValue({ set: mockSet, delete: mockDelete })
  })

  it('sets session, refresh, and expires cookies with HttpOnly', async () => {
    await setNucleusSession('tok_123', 'ref_456', '2026-12-31T00:00:00Z')

    expect(mockSet).toHaveBeenCalledTimes(3)

    const [sessionCall, refreshCall, expiresCall] = mockSet.mock.calls
    expect(sessionCall[0]).toBe('__nucleus_session')
    expect(sessionCall[1]).toBe('tok_123')
    expect(sessionCall[2]).toMatchObject({ httpOnly: true, secure: true, sameSite: 'lax', path: '/' })

    expect(refreshCall[0]).toBe('__nucleus_refresh')
    expect(refreshCall[1]).toBe('ref_456')
    expect(refreshCall[2].httpOnly).toBe(true)

    expect(expiresCall[0]).toBe('__nucleus_expires')
    expect(expiresCall[1]).toBe('2026-12-31T00:00:00Z')
    expect(expiresCall[2].httpOnly).toBe(true)
  })

  it('sets expiration date from expiresAt string', async () => {
    await setNucleusSession('tok', 'ref', '2026-06-15T12:00:00Z')

    const options = mockSet.mock.calls[0][2]
    expect(options.expires).toEqual(new Date('2026-06-15T12:00:00Z'))
  })
})

describe('clearNucleusSession', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockCookies.mockResolvedValue({ set: mockSet, delete: mockDelete })
  })

  it('deletes all three cookies', async () => {
    await clearNucleusSession()

    expect(mockDelete).toHaveBeenCalledTimes(3)
    expect(mockDelete).toHaveBeenCalledWith('__nucleus_session')
    expect(mockDelete).toHaveBeenCalledWith('__nucleus_refresh')
    expect(mockDelete).toHaveBeenCalledWith('__nucleus_expires')
  })
})

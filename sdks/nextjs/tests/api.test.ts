import { describe, it, expect, vi, beforeEach } from 'vitest'
import { NucleusApi } from '../src/client/api'

const mockFetch = vi.fn()
vi.stubGlobal('fetch', mockFetch)

function jsonResponse(data: unknown, status = 200) {
  return new Response(JSON.stringify(data), { status, headers: { 'Content-Type': 'application/json' } })
}

describe('NucleusApi', () => {
  let api: NucleusApi
  beforeEach(() => { vi.clearAllMocks(); api = new NucleusApi({ publishableKey: 'pk_test', baseUrl: 'https://api.test.com' }) })

  it('signIn sends POST', async () => {
    mockFetch.mockResolvedValue(jsonResponse({ user: { id: 'u' }, session: { id: 's' } }))
    await api.signIn('a@b.com', 'pass')
    expect(mockFetch.mock.calls[0][0]).toBe('https://api.test.com/v1/auth/sign-in')
  })

  it('signUp includes names', async () => {
    mockFetch.mockResolvedValue(jsonResponse({ user: {}, session: {} }))
    await api.signUp('a@b.com', 'p', 'J', 'D')
    expect(JSON.parse(mockFetch.mock.calls[0][1].body).first_name).toBe('J')
  })

  it('getUser returns user', async () => {
    mockFetch.mockResolvedValue(jsonResponse({ id: 'u_1' }))
    expect((await api.getUser('tok')).id).toBe('u_1')
  })

  it('refreshSession sends refresh_token', async () => {
    mockFetch.mockResolvedValue(jsonResponse({ id: 's', token: 'new' }))
    await api.refreshSession('ref')
    expect(JSON.parse(mockFetch.mock.calls[0][1].body).refresh_token).toBe('ref')
  })

  it('mfaTotpVerify sends code', async () => {
    mockFetch.mockResolvedValue(jsonResponse({ verified: true }))
    const r = await api.mfaTotpVerify('tok', '123456')
    expect(r.verified).toBe(true)
  })

  it('getOrganizations calls endpoint', async () => {
    mockFetch.mockResolvedValue(jsonResponse([]))
    await api.getOrganizations('tok')
    expect(mockFetch.mock.calls[0][0]).toContain('/v1/organizations')
  })

  it('throws on error', async () => {
    mockFetch.mockResolvedValue(new Response('fail', { status: 500 }))
    await expect(api.signIn('a', 'b')).rejects.toThrow()
  })

  it('204 returns undefined', async () => {
    mockFetch.mockResolvedValue(new Response(null, { status: 204 }))
    expect(await api.signOut('tok')).toBeUndefined()
  })
})

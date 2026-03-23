import { describe, it, expect, vi, beforeEach } from 'vitest'
import { NucleusApi } from '../src/client/api'

const mockFetch = vi.fn()
vi.stubGlobal('fetch', mockFetch)

function jsonResponse(data: unknown, status = 200) {
  return new Response(JSON.stringify(data), { status, headers: { 'Content-Type': 'application/json' } })
}

describe('NucleusApi', () => {
  let api: NucleusApi

  beforeEach(() => {
    vi.clearAllMocks()
    api = new NucleusApi({ publishableKey: 'pk_test', baseUrl: 'https://api.test.com' })
  })

  it('signIn sends correct request', async () => {
    mockFetch.mockResolvedValue(jsonResponse({ user: { id: 'u' }, session: { id: 's' } }))
    await api.signIn('a@b.com', 'password123')
    expect(mockFetch.mock.calls[0][0]).toBe('https://api.test.com/v1/auth/sign-in')
    expect(mockFetch.mock.calls[0][1].method).toBe('POST')
    expect(mockFetch.mock.calls[0][1].headers['X-Nucleus-Publishable-Key']).toBe('pk_test')
  })

  it('signUp sends names', async () => {
    mockFetch.mockResolvedValue(jsonResponse({ user: {}, session: {} }))
    await api.signUp('a@b.com', 'password123', 'J', 'D')
    const body = JSON.parse(mockFetch.mock.calls[0][1].body)
    expect(body.first_name).toBe('J')
  })

  it('signOut sends auth header', async () => {
    mockFetch.mockResolvedValue(new Response(null, { status: 204 }))
    await api.signOut('tok')
    expect(mockFetch.mock.calls[0][1].headers.Authorization).toBe('Bearer tok')
  })

  it('getUser returns user', async () => {
    mockFetch.mockResolvedValue(jsonResponse({ id: 'u_1', email: 'a@b.com' }))
    const user = await api.getUser('tok')
    expect(user.id).toBe('u_1')
  })

  it('refreshSession sends refresh_token', async () => {
    mockFetch.mockResolvedValue(jsonResponse({ id: 's', token: 'new' }))
    await api.refreshSession('ref_tok')
    const body = JSON.parse(mockFetch.mock.calls[0][1].body)
    expect(body.refresh_token).toBe('ref_tok')
  })

  it('mfaTotpSetup calls correct endpoint', async () => {
    mockFetch.mockResolvedValue(jsonResponse({ secret: 'ABC', qr_uri: 'otpauth://' }))
    await api.mfaTotpSetup('tok')
    expect(mockFetch.mock.calls[0][0]).toContain('/mfa/totp/setup')
  })

  it('getOrganizations calls correct endpoint', async () => {
    mockFetch.mockResolvedValue(jsonResponse([]))
    await api.getOrganizations('tok')
    expect(mockFetch.mock.calls[0][0]).toContain('/v1/organizations')
  })

  it('createInvitation sends correct body', async () => {
    mockFetch.mockResolvedValue(jsonResponse({ id: 'inv_1' }))
    await api.createInvitation('tok', 'org_1', 'x@y.com', 'admin')
    const body = JSON.parse(mockFetch.mock.calls[0][1].body)
    expect(body.email).toBe('x@y.com')
    expect(body.role).toBe('admin')
  })

  it('throws on error response', async () => {
    mockFetch.mockResolvedValue(new Response('Unauthorized', { status: 401 }))
    await expect(api.signIn('a@b.com', 'password123')).rejects.toThrow('Nucleus API error')
  })

  it('204 returns undefined', async () => {
    mockFetch.mockResolvedValue(new Response(null, { status: 204 }))
    const result = await api.signOut('tok')
    expect(result).toBeUndefined()
  })
})

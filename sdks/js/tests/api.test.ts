import { describe, it, expect, vi, beforeEach } from 'vitest'
import { NucleusApi, NucleusApiError } from '../src/api'

const mockFetch = vi.fn()
vi.stubGlobal('fetch', mockFetch)

function jsonResponse(data: unknown, status = 200) {
  return new Response(JSON.stringify(data), {
    status,
    headers: { 'Content-Type': 'application/json' },
  })
}

function errorResponse(status: number, body: string) {
  return new Response(body, { status, headers: {} })
}

describe('NucleusApi', () => {
  let api: NucleusApi

  beforeEach(() => {
    vi.clearAllMocks()
    api = new NucleusApi('pk_test_123', 'https://api.test.com')
  })

  describe('constructor', () => {
    it('strips trailing slash from baseUrl', () => {
      const a = new NucleusApi('pk', 'https://api.test.com/')
      mockFetch.mockResolvedValue(jsonResponse({ user: {}, session: {} }))
      a.signIn('a@b.com', 'password123')
      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.test.com/v1/auth/sign-in',
        expect.any(Object),
      )
    })

    it('uses default baseUrl when not provided', () => {
      const a = new NucleusApi('pk')
      mockFetch.mockResolvedValue(jsonResponse({ user: {}, session: {} }))
      a.signIn('a@b.com', 'password123')
      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.nucleus.dev/v1/auth/sign-in',
        expect.any(Object),
      )
    })
  })

  describe('signIn', () => {
    it('sends POST with email and password', async () => {
      const mockData = { user: { id: 'u_1', email: 'a@b.com' }, session: { id: 's_1', token: 'tok' } }
      mockFetch.mockResolvedValue(jsonResponse(mockData))

      const result = await api.signIn('a@b.com', 'password123')

      expect(mockFetch).toHaveBeenCalledWith('https://api.test.com/v1/auth/sign-in', {
        method: 'POST',
        body: JSON.stringify({ email: 'a@b.com', password: 'password123' }),
        headers: {
          'Content-Type': 'application/json',
          'X-Nucleus-Publishable-Key': 'pk_test_123',
        },
      })
      expect(result.user.id).toBe('u_1')
      expect(result.session.token).toBe('tok')
    })
  })

  describe('signUp', () => {
    it('sends POST with all fields', async () => {
      mockFetch.mockResolvedValue(jsonResponse({ user: { id: 'u_1' }, session: { id: 's_1' } }))
      await api.signUp('a@b.com', 'password123', 'John', 'Doe')
      const body = JSON.parse(mockFetch.mock.calls[0][1].body)
      expect(body).toEqual({ email: 'a@b.com', password: 'password123', first_name: 'John', last_name: 'Doe' })
    })
  })

  describe('signOut', () => {
    it('sends POST with auth header', async () => {
      mockFetch.mockResolvedValue(new Response(null, { status: 204 }))
      await api.signOut('tok_123')
      expect(mockFetch.mock.calls[0][1].headers.Authorization).toBe('Bearer tok_123')
      expect(mockFetch.mock.calls[0][1].method).toBe('POST')
    })
  })

  describe('getUser', () => {
    it('sends GET with auth header', async () => {
      mockFetch.mockResolvedValue(jsonResponse({ id: 'u_1', email: 'a@b.com' }))
      const user = await api.getUser('tok_123')
      expect(mockFetch.mock.calls[0][0]).toBe('https://api.test.com/v1/user')
      expect(mockFetch.mock.calls[0][1].headers.Authorization).toBe('Bearer tok_123')
      expect(user.id).toBe('u_1')
    })
  })

  describe('refreshSession', () => {
    it('sends POST with refresh_token', async () => {
      mockFetch.mockResolvedValue(jsonResponse({ id: 's_2', token: 'new_tok' }))
      const session = await api.refreshSession('ref_tok')
      const body = JSON.parse(mockFetch.mock.calls[0][1].body)
      expect(body.refresh_token).toBe('ref_tok')
      expect(session.token).toBe('new_tok')
    })
  })

  describe('getOrganizations', () => {
    it('sends GET with auth header', async () => {
      mockFetch.mockResolvedValue(jsonResponse([{ id: 'org_1', name: 'Test' }]))
      const orgs = await api.getOrganizations('tok')
      expect(mockFetch.mock.calls[0][0]).toBe('https://api.test.com/v1/organizations')
    })
  })

  describe('OAuth', () => {
    it('getOAuthUrl builds correct URL', () => {
      const url = api.getOAuthUrl('google', 'https://app.com/callback')
      expect(url).toContain('/v1/oauth/google/authorize')
      expect(url).toContain('redirect_uri=')
      expect(url).toContain('publishable_key=pk_test_123')
    })
  })

  describe('MFA', () => {
    it('mfaTotpSetup sends POST', async () => {
      mockFetch.mockResolvedValue(jsonResponse({ secret: 'ABCD', qr_uri: 'otpauth://...' }))
      const result = await api.mfaTotpSetup('tok')
      expect(mockFetch.mock.calls[0][0]).toContain('/v1/auth/mfa/totp/setup')
    })

    it('mfaTotpVerify sends code', async () => {
      mockFetch.mockResolvedValue(jsonResponse({ verified: true }))
      const result = await api.mfaTotpVerify('tok', '123456')
      const body = JSON.parse(mockFetch.mock.calls[0][1].body)
      expect(body.code).toBe('123456')
      expect(result.verified).toBe(true)
    })
  })

  describe('error handling', () => {
    it('throws NucleusApiError on non-ok response', async () => {
      mockFetch.mockResolvedValue(errorResponse(401, 'Unauthorized'))
      await expect(api.signIn('a@b.com', 'password123')).rejects.toThrow(NucleusApiError)
    })

    it('NucleusApiError contains status and body', async () => {
      mockFetch.mockResolvedValue(errorResponse(403, 'Forbidden'))
      try {
        await api.signIn('a@b.com', 'password123')
        expect.unreachable('should have thrown')
      } catch (e) {
        expect(e).toBeInstanceOf(NucleusApiError)
        expect((e as NucleusApiError).status).toBe(403)
        expect((e as NucleusApiError).body).toBe('Forbidden')
      }
    })

    it('returns undefined for 204 responses', async () => {
      mockFetch.mockResolvedValue(new Response(null, { status: 204 }))
      const result = await api.signOut('tok')
      expect(result).toBeUndefined()
    })
  })

  describe('organizations CRUD', () => {
    it('createOrganization sends name and slug', async () => {
      mockFetch.mockResolvedValue(jsonResponse({ id: 'org_1', name: 'Test', slug: 'test' }))
      await api.createOrganization('tok', 'Test', 'test')
      const body = JSON.parse(mockFetch.mock.calls[0][1].body)
      expect(body.name).toBe('Test')
      expect(body.slug).toBe('test')
    })

    it('createInvitation sends email and role', async () => {
      mockFetch.mockResolvedValue(jsonResponse({ id: 'inv_1' }))
      await api.createInvitation('tok', 'org_1', 'a@b.com', 'admin')
      const body = JSON.parse(mockFetch.mock.calls[0][1].body)
      expect(body.email).toBe('a@b.com')
      expect(body.role).toBe('admin')
    })
  })
})

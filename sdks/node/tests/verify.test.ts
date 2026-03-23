import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('jose', () => ({
  createRemoteJWKSet: vi.fn(() => 'mock-jwks'),
  jwtVerify: vi.fn(),
}))

import { verifyToken } from '../src/verify'
import * as jose from 'jose'

describe('verifyToken', () => {
  beforeEach(() => vi.clearAllMocks())

  it('creates JWKS from config baseUrl', async () => {
    vi.mocked(jose.jwtVerify).mockResolvedValue({ payload: { sub: 'u' }, protectedHeader: { alg: 'RS256' } } as any)
    await verifyToken('tok', { publishableKey: 'pk', secretKey: 'sk' })
    expect(jose.createRemoteJWKSet).toHaveBeenCalledWith(new URL('https://api.nucleus.dev/.well-known/jwks.json'))
  })

  it('uses custom baseUrl', async () => {
    vi.mocked(jose.jwtVerify).mockResolvedValue({ payload: { sub: 'u' }, protectedHeader: { alg: 'RS256' } } as any)
    await verifyToken('tok', { publishableKey: 'pk', secretKey: 'sk', baseUrl: 'https://custom.com' })
    expect(jose.createRemoteJWKSet).toHaveBeenCalledWith(new URL('https://custom.com/.well-known/jwks.json'))
  })

  it('caches JWKS — second call to same baseUrl does not create new JWKS', async () => {
    vi.mocked(jose.jwtVerify).mockResolvedValue({ payload: { sub: 'u' }, protectedHeader: { alg: 'RS256' } } as any)
    const config = { publishableKey: 'pk', secretKey: 'sk', baseUrl: 'https://cache-test.com' }
    await verifyToken('t1', config)
    const countAfterFirst = vi.mocked(jose.createRemoteJWKSet).mock.calls.length
    await verifyToken('t2', config)
    const countAfterSecond = vi.mocked(jose.createRemoteJWKSet).mock.calls.length
    expect(countAfterSecond).toBe(countAfterFirst) // no additional call
  })

  it('verifies with RS256', async () => {
    vi.mocked(jose.jwtVerify).mockResolvedValue({ payload: { sub: 'u' }, protectedHeader: { alg: 'RS256' } } as any)
    await verifyToken('tok', { publishableKey: 'pk', secretKey: 'sk' })
    expect(jose.jwtVerify).toHaveBeenCalledWith('tok', 'mock-jwks', { algorithms: ['RS256'] })
  })

  it('returns claims from payload', async () => {
    vi.mocked(jose.jwtVerify).mockResolvedValue({ payload: { sub: 'u_1', email: 'a@b.com', org_id: 'o_1' }, protectedHeader: { alg: 'RS256' } } as any)
    const claims = await verifyToken('tok', { publishableKey: 'pk', secretKey: 'sk' })
    expect(claims.sub).toBe('u_1')
    expect(claims.email).toBe('a@b.com')
  })

  it('throws on verification failure', async () => {
    vi.mocked(jose.jwtVerify).mockRejectedValue(new Error('invalid'))
    await expect(verifyToken('bad', { publishableKey: 'pk', secretKey: 'sk' })).rejects.toThrow()
  })
})

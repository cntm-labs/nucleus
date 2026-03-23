import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('jose', () => ({
  createRemoteJWKSet: vi.fn(() => 'mock-jwks'),
  jwtVerify: vi.fn(),
}))

import { verifyNucleusToken } from '../src/server/token'
import * as jose from 'jose'

describe('verifyNucleusToken', () => {
  beforeEach(() => vi.clearAllMocks())

  it('creates JWKS from baseUrl', async () => {
    vi.mocked(jose.jwtVerify).mockResolvedValue({ payload: { sub: 'u' }, protectedHeader: { alg: 'RS256' } } as any)
    await verifyNucleusToken('tok', 'https://api.test.com')
    expect(jose.createRemoteJWKSet).toHaveBeenCalledWith(new URL('https://api.test.com/.well-known/jwks.json'))
  })

  it('caches JWKS per baseUrl', async () => {
    vi.mocked(jose.jwtVerify).mockResolvedValue({ payload: { sub: 'u' }, protectedHeader: { alg: 'RS256' } } as any)
    await verifyNucleusToken('tok1', 'https://a.com')
    await verifyNucleusToken('tok2', 'https://a.com')
    expect(jose.createRemoteJWKSet).toHaveBeenCalledTimes(1)
  })

  it('uses different JWKS for different baseUrls', async () => {
    vi.mocked(jose.jwtVerify).mockResolvedValue({ payload: { sub: 'u' }, protectedHeader: { alg: 'RS256' } } as any)
    await verifyNucleusToken('tok1', 'https://unique-a.com')
    await verifyNucleusToken('tok2', 'https://unique-b.com')
    // Both URLs should have their own JWKS entry
    expect(jose.createRemoteJWKSet).toHaveBeenCalledWith(new URL('https://unique-a.com/.well-known/jwks.json'))
    expect(jose.createRemoteJWKSet).toHaveBeenCalledWith(new URL('https://unique-b.com/.well-known/jwks.json'))
  })

  it('verifies with RS256', async () => {
    vi.mocked(jose.jwtVerify).mockResolvedValue({ payload: { sub: 'u' }, protectedHeader: { alg: 'RS256' } } as any)
    await verifyNucleusToken('tok', 'https://a.com')
    expect(jose.jwtVerify).toHaveBeenCalledWith('tok', 'mock-jwks', { algorithms: ['RS256'] })
  })

  it('returns claims', async () => {
    vi.mocked(jose.jwtVerify).mockResolvedValue({ payload: { sub: 'user_1', email: 'a@b.com' }, protectedHeader: { alg: 'RS256' } } as any)
    const claims = await verifyNucleusToken('tok', 'https://a.com')
    expect(claims.sub).toBe('user_1')
    expect(claims.email).toBe('a@b.com')
  })

  it('throws on failure', async () => {
    vi.mocked(jose.jwtVerify).mockRejectedValue(new Error('bad'))
    await expect(verifyNucleusToken('bad', 'https://a.com')).rejects.toThrow()
  })
})

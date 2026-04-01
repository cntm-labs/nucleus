import { describe, it, expect, vi, beforeEach } from 'vitest'
import { NextRequest } from 'next/server'

// Hoist mock for token verification
const { mockVerify } = vi.hoisted(() => ({
  mockVerify: vi.fn(),
}))

vi.mock('../src/server/token', () => ({
  verifyNucleusToken: (...args: any[]) => mockVerify(...args),
}))

import { authMiddleware } from '../src/server/middleware'

function makeRequest(path: string, sessionToken?: string): NextRequest {
  const url = `http://localhost${path}`
  const headers = new Headers()
  if (sessionToken) {
    headers.set('cookie', `__nucleus_session=${sessionToken}`)
  }
  return new NextRequest(url, { headers })
}

describe('authMiddleware', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('allows requests to public routes without token', async () => {
    const mw = authMiddleware({ publicRoutes: ['/sign-in', '/sign-up'] })
    const req = makeRequest('/sign-in')

    const res = await mw(req)

    // NextResponse.next() returns status 200
    expect(res.status).toBe(200)
  })

  it('redirects unauthenticated requests to sign-in', async () => {
    const mw = authMiddleware({ publicRoutes: ['/sign-in'] })
    const req = makeRequest('/dashboard')

    const res = await mw(req)

    expect(res.status).toBe(307)
    const location = new URL(res.headers.get('location')!)
    expect(location.pathname).toBe('/sign-in')
    expect(location.searchParams.get('redirect_url')).toBe('/dashboard')
  })

  it('allows authenticated requests through', async () => {
    mockVerify.mockResolvedValue({ sub: 'user_123', org_id: null })

    const mw = authMiddleware({ publicRoutes: [] })
    const req = makeRequest('/dashboard', 'valid.jwt.token')

    const res = await mw(req)

    expect(mockVerify).toHaveBeenCalledWith('valid.jwt.token', expect.any(String))
    expect(res.status).toBe(200)
  })

  it('redirects when token is invalid on protected route', async () => {
    mockVerify.mockRejectedValue(new Error('token expired'))

    const mw = authMiddleware({ publicRoutes: [] })
    const req = makeRequest('/dashboard', 'expired.jwt')

    const res = await mw(req)

    expect(res.status).toBe(307)
  })

  it('allows invalid token on public route', async () => {
    mockVerify.mockRejectedValue(new Error('token expired'))

    const mw = authMiddleware({ publicRoutes: ['/sign-in'] })
    const req = makeRequest('/sign-in', 'expired.jwt')

    const res = await mw(req)

    // Should still allow the public route
    expect(res.status).toBe(200)
  })

  it('supports wildcard public routes', async () => {
    const mw = authMiddleware({ publicRoutes: ['/auth(.*)'] })
    const req = makeRequest('/auth/callback')

    const res = await mw(req)

    expect(res.status).toBe(200)
  })

  it('calls afterAuth callback with userId and orgId', async () => {
    mockVerify.mockResolvedValue({ sub: 'user_123', org_id: 'org_456' })

    const afterAuth = vi.fn()
    const mw = authMiddleware({ publicRoutes: [], afterAuth })
    const req = makeRequest('/dashboard', 'valid.jwt')

    await mw(req)

    expect(afterAuth).toHaveBeenCalledWith(
      { userId: 'user_123', orgId: 'org_456' },
      expect.anything()
    )
  })

  it('uses custom signInUrl for redirects', async () => {
    const mw = authMiddleware({ publicRoutes: [], signInUrl: '/login' })
    const req = makeRequest('/protected')

    const res = await mw(req)

    const location = new URL(res.headers.get('location')!)
    expect(location.pathname).toBe('/login')
  })
})

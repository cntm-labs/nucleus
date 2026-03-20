import { NextRequest, NextResponse } from 'next/server'
import { verifyNucleusToken } from './token'

export interface AuthMiddlewareOptions {
  publicRoutes?: string[]
  signInUrl?: string
  baseUrl?: string
  afterAuth?: (auth: { userId: string | null; orgId: string | null }, req: NextRequest) => NextResponse | void
}

const NUCLEUS_SESSION_COOKIE = '__nucleus_session'

export function authMiddleware(options: AuthMiddlewareOptions = {}) {
  const {
    publicRoutes = [],
    signInUrl = '/sign-in',
    baseUrl = process.env.NUCLEUS_API_URL ?? 'https://api.nucleus.dev',
  } = options

  return async (req: NextRequest) => {
    const { pathname } = req.nextUrl

    // Check if route is public
    const isPublic = publicRoutes.some(route => {
      if (route.endsWith('(.*)')) {
        return pathname.startsWith(route.replace('(.*)', ''))
      }
      return pathname === route || pathname.match(new RegExp(`^${route}$`))
    })

    const token = req.cookies.get(NUCLEUS_SESSION_COOKIE)?.value
    let userId: string | null = null
    let orgId: string | null = null

    if (token) {
      try {
        const claims = await verifyNucleusToken(token, baseUrl)
        userId = claims.sub
        orgId = claims.org_id ?? null
      } catch {
        // Token invalid — treat as unauthenticated
        const response = isPublic ? NextResponse.next() : NextResponse.redirect(new URL(signInUrl, req.url))
        response.cookies.delete(NUCLEUS_SESSION_COOKIE)
        return response
      }
    }

    // Custom afterAuth handler
    if (options.afterAuth) {
      const result = options.afterAuth({ userId, orgId }, req)
      if (result) return result
    }

    // Redirect unauthenticated users from protected routes
    if (!isPublic && !userId) {
      const url = new URL(signInUrl, req.url)
      url.searchParams.set('redirect_url', pathname)
      return NextResponse.redirect(url)
    }

    return NextResponse.next()
  }
}

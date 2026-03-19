import { NextRequest, NextResponse } from 'next/server'

export interface AuthMiddlewareOptions {
  publicRoutes?: string[]
  afterAuth?: (auth: { userId: string | null; orgId: string | null }, req: NextRequest) => void
}

export function authMiddleware(options: AuthMiddlewareOptions = {}) {
  return async (req: NextRequest) => {
    const isPublic = options.publicRoutes?.some(route =>
      req.nextUrl.pathname.match(new RegExp(route))
    )
    if (isPublic) return NextResponse.next()

    // TODO: verify JWT from cookies
    return NextResponse.next()
  }
}

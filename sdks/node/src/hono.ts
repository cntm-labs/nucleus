import { verifyToken } from './verify'
import { NucleusConfig, NucleusClaims } from './types'

export function nucleusMiddleware(config: NucleusConfig) {
  return async (c: any, next: () => Promise<void>) => {
    const authHeader = c.req.header('authorization')
    if (authHeader?.startsWith('Bearer ')) {
      try {
        const claims = await verifyToken(authHeader.slice(7), config)
        c.set('nucleusClaims', claims)
      } catch { /* continue without claims */ }
    }
    await next()
  }
}

export type { NucleusConfig, NucleusClaims }

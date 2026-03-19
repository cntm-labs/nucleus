import { verifyToken } from './verify'
import { NucleusConfig, NucleusClaims } from './types'

export function nucleusPlugin(config: NucleusConfig) {
  return async function (fastify: any) {
    fastify.decorateRequest('nucleusClaims', null)
    fastify.addHook('onRequest', async (request: any) => {
      const authHeader = request.headers.authorization
      if (!authHeader?.startsWith('Bearer ')) return
      try {
        request.nucleusClaims = await verifyToken(authHeader.slice(7), config)
      } catch { /* continue without claims */ }
    })
  }
}

export type { NucleusConfig, NucleusClaims }

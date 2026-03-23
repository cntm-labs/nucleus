import * as jose from 'jose'
import { NucleusClaims, NucleusConfig } from './types'

const jwksCache = new Map<string, ReturnType<typeof jose.createRemoteJWKSet>>()

export async function verifyToken(
  token: string,
  config: NucleusConfig
): Promise<NucleusClaims> {
  const baseUrl = (config.baseUrl || 'https://api.nucleus.dev').replace(/\/$/, '')
  let jwks = jwksCache.get(baseUrl)
  if (!jwks) {
    jwks = jose.createRemoteJWKSet(new URL(`${baseUrl}/.well-known/jwks.json`))
    jwksCache.set(baseUrl, jwks)
  }
  const { payload } = await jose.jwtVerify(token, jwks, {
    algorithms: ['RS256'],
  })
  return payload as unknown as NucleusClaims
}

import * as jose from 'jose'
import type { NucleusClaims } from '../client/types'

const jwksCache = new Map<string, ReturnType<typeof jose.createRemoteJWKSet>>()

export async function verifyNucleusToken(token: string, baseUrl: string): Promise<NucleusClaims> {
  let jwks = jwksCache.get(baseUrl)
  if (!jwks) {
    jwks = jose.createRemoteJWKSet(new URL(`${baseUrl}/.well-known/jwks.json`))
    jwksCache.set(baseUrl, jwks)
  }
  const { payload } = await jose.jwtVerify(token, jwks, { algorithms: ['RS256'] })
  return payload as unknown as NucleusClaims
}

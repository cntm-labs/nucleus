import * as jose from 'jose'
import type { NucleusClaims } from '../client/types'

let jwks: ReturnType<typeof jose.createRemoteJWKSet> | null = null

export async function verifyNucleusToken(token: string, baseUrl: string): Promise<NucleusClaims> {
  if (!jwks) {
    jwks = jose.createRemoteJWKSet(new URL(`${baseUrl}/.well-known/jwks.json`))
  }
  const { payload } = await jose.jwtVerify(token, jwks, { algorithms: ['RS256'] })
  return payload as unknown as NucleusClaims
}

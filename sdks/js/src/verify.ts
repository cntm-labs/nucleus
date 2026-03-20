import * as jose from 'jose'
import { NucleusClaims } from './types'
export async function verifyToken(token: string, baseUrl: string): Promise<NucleusClaims> {
  const JWKS = jose.createRemoteJWKSet(new URL(`${baseUrl}/.well-known/jwks.json`))
  const { payload } = await jose.jwtVerify(token, JWKS, { algorithms: ['RS256'] })
  return payload as unknown as NucleusClaims
}

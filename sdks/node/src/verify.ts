import * as jose from 'jose'
import { NucleusClaims, NucleusConfig } from './types'

let cachedJwks: jose.JSONWebKeySet | null = null
let cacheExpiry = 0

export async function verifyToken(
  token: string,
  config: NucleusConfig
): Promise<NucleusClaims> {
  const baseUrl = config.baseUrl || 'https://api.nucleus.dev'
  const JWKS = jose.createRemoteJWKSet(new URL(`${baseUrl}/.well-known/jwks.json`))
  const { payload } = await jose.jwtVerify(token, JWKS, {
    algorithms: ['RS256'],
  })
  return payload as unknown as NucleusClaims
}

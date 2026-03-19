import * as jose from 'jose'

export async function verifyNucleusToken(req: Request) {
  const auth = req.headers.get('authorization')
  if (!auth?.startsWith('Bearer ')) throw new Error('No token')

  const token = auth.slice(7)
  void jose
  void token

  // TODO: verify with JWKS
  return {} as Record<string, unknown>
}

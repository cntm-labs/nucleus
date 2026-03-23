import { cookies } from 'next/headers'
import { verifyNucleusToken } from './token'
import type { NucleusClaims } from '../client/types'

const NUCLEUS_SESSION_COOKIE = '__nucleus_session'

function getBaseUrl(): string {
  return process.env.NUCLEUS_API_URL ?? 'https://api.nucleus.dev'
}

export async function currentUser(): Promise<NucleusClaims | null> {
  try {
    const cookieStore = await cookies()
    const token = cookieStore.get(NUCLEUS_SESSION_COOKIE)?.value
    if (!token) return null
    return await verifyNucleusToken(token, getBaseUrl())
  } catch {
    return null
  }
}

export async function auth(): Promise<{
  userId: string | null
  orgId: string | null
  orgRole: string | null
  orgPermissions: string[]
  claims: NucleusClaims | null
  getToken: () => Promise<string | null>
}> {
  const claims = await currentUser()
  const cookieStore = await cookies()
  const token = cookieStore.get(NUCLEUS_SESSION_COOKIE)?.value ?? null

  return {
    userId: claims?.sub ?? null,
    orgId: claims?.org_id ?? null,
    orgRole: claims?.org_role ?? null,
    orgPermissions: claims?.org_permissions ?? [],
    claims,
    getToken: async () => claims ? token : null,
  }
}

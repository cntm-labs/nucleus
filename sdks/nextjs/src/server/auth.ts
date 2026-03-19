import * as jose from 'jose'

export async function currentUser() {
  // TODO: read from cookies/headers and verify JWT
  void jose
  return null
}

export function auth() {
  return {
    userId: null as string | null,
    orgId: null as string | null,
    getToken: async () => null as string | null,
  }
}

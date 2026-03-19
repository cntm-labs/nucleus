import type { NucleusUser, NucleusSession, NucleusOrganization } from './types'

const DEFAULT_BASE_URL = 'https://api.nucleus.auth'

interface NucleusApiOptions {
  publishableKey: string
  baseUrl?: string
}

export class NucleusApi {
  private publishableKey: string
  private baseUrl: string

  constructor(options: NucleusApiOptions) {
    this.publishableKey = options.publishableKey
    this.baseUrl = options.baseUrl ?? DEFAULT_BASE_URL
  }

  private async request<T>(path: string, options: RequestInit = {}): Promise<T> {
    const url = `${this.baseUrl}${path}`
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        'X-Nucleus-Publishable-Key': this.publishableKey,
        ...options.headers,
      },
    })

    if (!response.ok) {
      const body = await response.text()
      throw new Error(`Nucleus API error (${response.status}): ${body}`)
    }

    return response.json() as Promise<T>
  }

  async signIn(email: string, password: string): Promise<{ user: NucleusUser; session: NucleusSession }> {
    return this.request('/v1/auth/sign-in', {
      method: 'POST',
      body: JSON.stringify({ email, password }),
    })
  }

  async signUp(email: string, password: string, firstName?: string, lastName?: string): Promise<{ user: NucleusUser; session: NucleusSession }> {
    return this.request('/v1/auth/sign-up', {
      method: 'POST',
      body: JSON.stringify({ email, password, first_name: firstName, last_name: lastName }),
    })
  }

  async signOut(token: string): Promise<void> {
    await this.request('/v1/auth/sign-out', {
      method: 'POST',
      headers: { Authorization: `Bearer ${token}` },
    })
  }

  async getUser(token: string): Promise<NucleusUser> {
    return this.request('/v1/user', {
      headers: { Authorization: `Bearer ${token}` },
    })
  }

  async getSession(token: string): Promise<NucleusSession> {
    return this.request('/v1/session', {
      headers: { Authorization: `Bearer ${token}` },
    })
  }

  async refreshSession(token: string): Promise<NucleusSession> {
    return this.request('/v1/session/refresh', {
      method: 'POST',
      headers: { Authorization: `Bearer ${token}` },
    })
  }

  async getOrganizations(token: string): Promise<NucleusOrganization[]> {
    return this.request('/v1/organizations', {
      headers: { Authorization: `Bearer ${token}` },
    })
  }
}

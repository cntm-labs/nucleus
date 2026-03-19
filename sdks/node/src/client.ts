import { NucleusConfig, NucleusUser, PaginatedResponse, ListUsersParams } from './types'

export class NucleusClient {
  private baseUrl: string
  private secretKey: string

  constructor(config: NucleusConfig) {
    this.secretKey = config.secretKey
    this.baseUrl = config.baseUrl || 'https://api.nucleus.dev'
  }

  get users() { return new UsersApi(this.baseUrl, this.secretKey) }
  get orgs() { return new OrgsApi(this.baseUrl, this.secretKey) }
}

class BaseApi {
  constructor(protected baseUrl: string, protected secretKey: string) {}

  protected async request<T>(path: string, options: RequestInit = {}): Promise<T> {
    const res = await fetch(`${this.baseUrl}/api/v1/admin${path}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${this.secretKey}`,
        ...options.headers,
      },
    })
    if (!res.ok) {
      const error = await res.json().catch(() => ({}))
      throw new Error(error.error?.message || `API error: ${res.status}`)
    }
    return res.json()
  }
}

class UsersApi extends BaseApi {
  async get(userId: string): Promise<NucleusUser> {
    return this.request(`/users/${userId}`)
  }
  async list(params?: ListUsersParams): Promise<PaginatedResponse<NucleusUser>> {
    const query = new URLSearchParams()
    if (params?.limit) query.set('limit', String(params.limit))
    if (params?.cursor) query.set('cursor', params.cursor)
    if (params?.email_contains) query.set('email_contains', params.email_contains)
    return this.request(`/users?${query}`)
  }
  async delete(userId: string): Promise<void> {
    await this.request(`/users/${userId}`, { method: 'DELETE' })
  }
  async ban(userId: string): Promise<void> {
    await this.request(`/users/${userId}/ban`, { method: 'POST' })
  }
  async unban(userId: string): Promise<void> {
    await this.request(`/users/${userId}/unban`, { method: 'POST' })
  }
}

class OrgsApi extends BaseApi {
  async get(orgId: string) { return this.request(`/orgs/${orgId}`) }
  async list(params?: { limit?: number; cursor?: string }) {
    const query = new URLSearchParams()
    if (params?.limit) query.set('limit', String(params.limit))
    if (params?.cursor) query.set('cursor', params.cursor)
    return this.request(`/orgs?${query}`)
  }
}

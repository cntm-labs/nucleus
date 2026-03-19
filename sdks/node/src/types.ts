export interface NucleusConfig {
  secretKey: string
  baseUrl?: string
  jwksCacheTtlMs?: number
}

export interface NucleusClaims {
  sub: string          // user_id
  iss: string          // issuer
  aud: string          // project_id
  exp: number
  iat: number
  jti: string
  email?: string
  first_name?: string
  last_name?: string
  avatar_url?: string
  email_verified?: boolean
  metadata?: Record<string, unknown>
  org_id?: string
  org_slug?: string
  org_role?: string
  org_permissions?: string[]
}

export interface NucleusUser {
  id: string
  email: string
  email_verified: boolean
  username?: string
  first_name?: string
  last_name?: string
  avatar_url?: string
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface PaginatedResponse<T> {
  data: T[]
  has_more: boolean
  next_cursor?: string
  total_count?: number
}

export interface ListUsersParams {
  limit?: number
  cursor?: string
  email_contains?: string
}

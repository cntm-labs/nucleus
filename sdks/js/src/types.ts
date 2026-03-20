export interface NucleusConfig {
  publishableKey: string
  baseUrl?: string
}

export interface NucleusUser {
  id: string
  email: string
  email_verified: boolean
  phone?: string
  phone_verified: boolean
  first_name?: string
  last_name?: string
  avatar_url?: string
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface NucleusSession {
  id: string
  token: string
  refresh_token: string
  expires_at: string
  user_id: string
}

export interface NucleusOrganization {
  id: string
  name: string
  slug: string
  created_at: string
}

export interface NucleusMember {
  id: string
  user_id: string
  org_id: string
  role: string
  permissions: string[]
  email: string
  first_name?: string
  last_name?: string
}

export interface NucleusInvitation {
  id: string
  org_id: string
  email: string
  role: string
  status: 'pending' | 'accepted' | 'revoked'
  created_at: string
}

export interface NucleusClaims {
  sub: string
  iss: string
  aud: string
  exp: number
  iat: number
  jti: string
  email: string
  first_name?: string
  last_name?: string
  avatar_url?: string
  email_verified: boolean
  metadata: Record<string, unknown>
  org_id?: string
  org_slug?: string
  org_role?: string
  org_permissions?: string[]
}

export interface NucleusMfaSetup {
  secret: string
  qr_uri: string
}

export interface NucleusAuthResponse {
  user: NucleusUser
  session: NucleusSession
}

export type OAuthProvider = 'google' | 'github' | 'apple' | 'microsoft' | 'discord' | 'slack'

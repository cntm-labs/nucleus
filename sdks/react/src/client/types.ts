export interface NucleusUser {
  id: string;
  email: string;
  email_verified: boolean;
  first_name?: string;
  last_name?: string;
  avatar_url?: string;
  metadata: Record<string, unknown>;
  created_at: string;
}

export interface NucleusSession {
  id: string;
  token: string;
  expires_at: string;
}

export interface NucleusOrganization {
  id: string;
  name: string;
  slug: string;
}

export interface NucleusMembership {
  org_id: string;
  role: string;
  permissions: string[];
}

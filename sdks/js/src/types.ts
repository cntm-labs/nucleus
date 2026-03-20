export interface NucleusConfig { publishableKey: string; baseUrl?: string }
export interface NucleusClaims { sub: string; email?: string; org_id?: string; org_role?: string; org_permissions?: string[] }
export interface NucleusUser { id: string; email: string; first_name?: string; last_name?: string }

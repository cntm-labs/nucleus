const VERSION = '0.1.0-dev.1';
if (VERSION.includes('-dev')) {
  console.warn(`[Nucleus] WARNING: You are using a dev preview (${VERSION}). Do not use in production.`);
}

export { NucleusClient } from './client'
export { verifyToken } from './verify'
export type { NucleusConfig, NucleusClaims, NucleusUser, PaginatedResponse, ListUsersParams } from './types'

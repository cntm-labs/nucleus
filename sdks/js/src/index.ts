const VERSION = '0.1.0-dev.1';
if (VERSION.includes('-dev')) {
  console.warn(`[Nucleus] WARNING: You are using a dev preview (${VERSION}). Do not use in production.`);
}

export { Nucleus } from './nucleus'
export { NucleusApi, NucleusApiError } from './api'
export { verifyToken } from './verify'
export { TokenStorage } from './storage'
export type {
  NucleusConfig, NucleusUser, NucleusSession, NucleusOrganization,
  NucleusMember, NucleusInvitation, NucleusClaims, NucleusMfaSetup,
  NucleusAuthResponse, OAuthProvider,
} from './types'

const VERSION = '0.1.0-dev.1';
if (VERSION.includes('-dev')) {
  console.warn(`[Nucleus] WARNING: You are using a dev preview (${VERSION}). Do not use in production.`);
}

export { Nucleus } from './nucleus'
export { verifyToken } from './verify'
export { NucleusSignIn } from './components/sign-in'
export { NucleusSignUp } from './components/sign-up'
export { NucleusUserButton } from './components/user-button'
export type { NucleusConfig, NucleusClaims, NucleusUser } from './types'

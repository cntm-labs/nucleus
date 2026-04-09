const VERSION = '0.1.0-dev.1';
if (VERSION.includes('-dev')) {
  console.warn(`[Nucleus] WARNING: You are using a dev preview (${VERSION}). Do not use in production.`);
}

// Provider
export { NucleusProvider, useNucleus } from './provider'
export type { NucleusProviderProps } from './provider'

// Types
export type {
  NucleusUser, NucleusSession, NucleusOrganization,
  NucleusMember, NucleusInvitation, NucleusMfaSetup,
  NucleusAuthResponse, NucleusClaims, OAuthProvider,
} from './client/types'

// Hooks
export { useUser } from './hooks/use-user'
export { useSession } from './hooks/use-session'
export { useAuth } from './hooks/use-auth'
export { useSignIn } from './hooks/use-sign-in'
export { useSignUp } from './hooks/use-sign-up'
export { useOrganization } from './hooks/use-organization'
export { useOrganizationList } from './hooks/use-organization-list'
export { useOAuth } from './hooks/use-oauth'
export { usePasskey } from './hooks/use-passkey'
export { useMfa } from './hooks/use-mfa'
export { useVerification } from './hooks/use-verification'
export { useProfile } from './hooks/use-profile'
export { useSessionList } from './hooks/use-session-list'

// Components
export { SignIn } from './components/sign-in'
export type { SignInProps } from './components/sign-in'
export { SignUp } from './components/sign-up'
export type { SignUpProps } from './components/sign-up'
export { UserButton } from './components/user-button'
export type { UserButtonProps } from './components/user-button'
export { UserProfile } from './components/user-profile'
export type { UserProfileProps } from './components/user-profile'
export { OrgSwitcher } from './components/org-switcher'
export type { OrgSwitcherProps } from './components/org-switcher'
export { OrgProfile } from './components/org-profile'
export type { OrgProfileProps } from './components/org-profile'

// i18n
export { useTranslation, I18nContext } from './i18n'
export type { Locale } from './i18n'
export { en } from './i18n/locales/en'
export { th } from './i18n/locales/th'

// Client utilities
export { NucleusApi } from './client/api'
export { getSessionToken, getRefreshToken, getExpiresAt, setSessionTokens, clearSessionTokens } from './client/session'

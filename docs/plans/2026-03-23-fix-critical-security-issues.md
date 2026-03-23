# Fix Critical Security & Quality Issues — Production Readiness

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix 6 critical issues blocking production readiness across all SDKs — security, correctness, and SSR safety.

**Architecture:** Each fix is isolated per-SDK. OAuth CSRF state is the most cross-cutting (affects JS, React, Next.js, Flutter). Other fixes are SDK-specific. All fixes include tests.

**Tech Stack:** TypeScript (Vitest), Dart (flutter_test), WebCrypto (crypto.randomUUID for state)

**Execution order:** OAuth CSRF → React MFA → JS SSR → Next.js HttpOnly → Flutter OAuth → Input Validation

---

## Task 1: OAuth CSRF state parameter (JS, React, Next.js, Flutter)

### Problem
OAuth flows don't include a `state` parameter — vulnerable to CSRF attacks. Any page on the same origin can send a fake `nucleus:oauth:callback` message.

### Task 1.1: Fix JS SDK OAuth

**Files:**
- Modify: `sdks/js/src/oauth.ts`
- Test: `sdks/js/tests/oauth.test.ts`

**Step 1: Write failing test**

```typescript
// sdks/js/tests/oauth.test.ts
import { describe, it, expect, vi } from 'vitest'

describe('OAuth CSRF', () => {
  it('generates a random state parameter', () => {
    // State should be included in the OAuth URL
  })
  it('rejects callback with mismatched state', () => {
    // Callback without correct state should be rejected
  })
})
```

**Step 2: Fix oauth.ts — add state generation + validation**

```typescript
// Generate state before opening popup
const state = crypto.randomUUID()
sessionStorage.setItem('__nucleus_oauth_state', state)

// Include state in OAuth URL
const url = api.getOAuthUrl(provider, redirectUri) + `&state=${state}`

// In onMessage handler, validate state
if (event.data.state !== sessionStorage.getItem('__nucleus_oauth_state')) {
  reject(new Error('OAuth state mismatch — possible CSRF attack'))
  return
}
sessionStorage.removeItem('__nucleus_oauth_state')
```

**Step 3: Fix api.ts getOAuthUrl — accept state param**

Add optional `state` parameter to `getOAuthUrl()` method.

**Step 4: Verify test passes**

Run: `cd sdks/js && npx vitest run`

**Step 5: Commit**

```bash
git commit -m "fix(js): add OAuth CSRF state parameter"
```

### Task 1.2: Fix React SDK OAuth

**Files:**
- Modify: `sdks/react/src/hooks/use-oauth.ts`

Same pattern — generate `crypto.randomUUID()`, store in `sessionStorage`, validate on callback.

**Commit:** `fix(react): add OAuth CSRF state parameter`

### Task 1.3: Fix Next.js SDK OAuth

**Files:**
- Modify: `sdks/nextjs/src/hooks/use-oauth.ts`

Same pattern as React.

**Commit:** `fix(nextjs): add OAuth CSRF state parameter`

### Task 1.4: Fix Flutter SDK OAuth

**Files:**
- Modify: `sdks/flutter/lib/src/auth/oauth.dart`

Use `uuid` package or `DateTime.now().millisecondsSinceEpoch.toRadixString(36)` for state. Store in memory (not SharedPreferences — it's ephemeral). Validate on callback.

**Commit:** `fix(flutter): add OAuth CSRF state parameter`

---

## Task 2: Fix React MFA submit (broken stub)

### Problem
`<SignIn>` component collects MFA code but the form `onSubmit` just calls `setStep('credentials')` — never sends the code to the server.

**Files:**
- Modify: `sdks/react/src/components/sign-in.tsx`
- Modify: `sdks/nextjs/src/components/sign-in.tsx` (same pattern)
- Test: `sdks/react/tests/components.test.tsx`

**Step 1: Write failing test**

```typescript
// Test that MFA form calls mfaTotpVerify when submitted
```

**Step 2: Fix sign-in.tsx**

Replace the MFA stub with real `useMfa()` hook integration:

```tsx
const { verifyTotp, isLoading: mfaLoading, error: mfaError } = useMfa()

// In MFA form onSubmit:
const result = await verifyTotp(mfaCode)
if (result.verified) {
  onSignIn?.()
  if (typeof window !== 'undefined') window.location.href = afterSignInUrl
}
```

**Step 3: Apply same fix to Next.js SignIn**

**Step 4: Test**

Run: `cd sdks/react && npx vitest run`

**Step 5: Commit**

```bash
git commit -m "fix(react,nextjs): wire MFA TOTP verification in SignIn component"
```

---

## Task 3: JS SDK SSR guards

### Problem
`storage.ts` calls `localStorage` directly without SSR guards. Crashes in Node.js SSR environments.

**Files:**
- Modify: `sdks/js/src/storage.ts`
- Modify: `sdks/js/src/oauth.ts` (uses `window`, `sessionStorage`)
- Modify: `sdks/js/src/passkey.ts` (uses `navigator`, `window`)
- Test: `sdks/js/tests/storage.test.ts`

**Step 1: Write failing test**

```typescript
// Test that TokenStorage methods return null/no-op in non-browser environment
```

**Step 2: Add SSR guards to storage.ts**

Wrap every `localStorage` call with `typeof window !== 'undefined'` check. Return `null` / no-op when server-side.

```typescript
getActiveToken(): string | null {
  if (typeof window === 'undefined') return null
  return localStorage.getItem(`${PREFIX}token`)
},
```

**Step 3: Add SSR guards to oauth.ts and passkey.ts**

Throw clear error: `throw new Error('OAuth is only available in browser environments')`

**Step 4: Test**

Run: `cd sdks/js && npx vitest run`

**Step 5: Commit**

```bash
git commit -m "fix(js): add SSR guards for localStorage, window, navigator"
```

---

## Task 4: Next.js server-side cookie setting via API Route

### Problem
Session cookies are set via `document.cookie` without `HttpOnly` flag. In production, cookies should be set server-side.

**Files:**
- Create: `sdks/nextjs/src/server/set-session.ts` — Server Action or API route helper
- Modify: `sdks/nextjs/src/client/session.ts` — add `setSessionViaServer()` option
- Test: `sdks/nextjs/tests/set-session.test.ts`

**Step 1: Create server-side session setter**

```typescript
// sdks/nextjs/src/server/set-session.ts
'use server'
import { cookies } from 'next/headers'

export async function setNucleusSession(token: string, refreshToken: string, expiresAt: string) {
  const cookieStore = await cookies()
  const expires = new Date(expiresAt)
  cookieStore.set('__nucleus_session', token, { httpOnly: true, secure: true, sameSite: 'lax', path: '/', expires })
  cookieStore.set('__nucleus_refresh', refreshToken, { httpOnly: true, secure: true, sameSite: 'lax', path: '/', expires })
  cookieStore.set('__nucleus_expires', expiresAt, { httpOnly: true, secure: true, sameSite: 'lax', path: '/', expires })
}

export async function clearNucleusSession() {
  const cookieStore = await cookies()
  cookieStore.delete('__nucleus_session')
  cookieStore.delete('__nucleus_refresh')
  cookieStore.delete('__nucleus_expires')
}
```

**Step 2: Export from server/index.ts**

**Step 3: Update documentation comment in client/session.ts**

Note that `setSessionTokens` is a client-side fallback; for production use `setNucleusSession` Server Action.

**Step 4: Test**

Write test mocking `next/headers` cookies.

**Step 5: Commit**

```bash
git commit -m "feat(nextjs): add server-side HttpOnly cookie setter via Server Action"
```

---

## Task 5: Flutter OAuth wiring

### Problem
`NucleusOAuth` exists but is not wired into `NucleusAuth`. There's no `signInWithOAuth()` on the auth state.

**Files:**
- Modify: `sdks/flutter/lib/src/auth/auth_state.dart` — add `signInWithOAuth`
- Modify: `sdks/flutter/lib/src/nucleus.dart` — expose OAuth manager
- Test: `sdks/flutter/test/auth_state_test.dart`

**Step 1: Add signInWithOAuth to NucleusAuth**

```dart
Future<void> signInWithOAuth(String provider, {String redirectUri = 'nucleus://oauth/callback'}) async {
  final oauth = NucleusOAuth(_api);
  await oauth.launchOAuth(/* need context */, provider, redirectUri: redirectUri);
}

Future<void> handleOAuthCallback(String code, {String redirectUri = 'nucleus://oauth/callback'}) async {
  final oauth = NucleusOAuth(_api);
  final result = await oauth.handleCallback(code, redirectUri: redirectUri);
  await _setAuthResult(result.user, result.session);
}
```

**Step 2: Expose via Nucleus singleton**

```dart
static NucleusOAuth get oauth => NucleusOAuth(client);
```

**Step 3: Test**

Write test for handleOAuthCallback state update.

**Step 4: Commit**

```bash
git commit -m "feat(flutter): wire OAuth flow into NucleusAuth"
```

---

## Task 6: Input validation

### Problem
No client-side validation — empty strings, invalid emails, short passwords are sent directly to the API.

**Files:**
- Create: `sdks/js/src/validation.ts`
- Modify: `sdks/js/src/api.ts` — validate before request
- Create: `sdks/react/src/client/validation.ts` (copy from JS)
- Create: `sdks/nextjs/src/client/validation.ts` (copy from JS)
- Create: `sdks/flutter/lib/src/validation.dart`
- Test: `sdks/js/tests/validation.test.ts`

**Step 1: Create shared validation module**

```typescript
// sdks/js/src/validation.ts
export class ValidationError extends Error {
  constructor(public field: string, message: string) {
    super(message)
    this.name = 'ValidationError'
  }
}

export function validateEmail(email: string): void {
  if (!email || email.trim().length === 0) throw new ValidationError('email', 'Email is required')
  if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) throw new ValidationError('email', 'Invalid email format')
}

export function validatePassword(password: string): void {
  if (!password || password.length === 0) throw new ValidationError('password', 'Password is required')
  if (password.length < 8) throw new ValidationError('password', 'Password must be at least 8 characters')
}

export function validateRequired(value: string | undefined | null, field: string): void {
  if (!value || value.trim().length === 0) throw new ValidationError(field, `${field} is required`)
}
```

**Step 2: Add validation to signIn/signUp in api.ts**

```typescript
signIn(email: string, password: string) {
  validateEmail(email)
  validatePassword(password)
  return this.request<NucleusAuthResponse>(...)
}
```

**Step 3: Write tests**

```typescript
// sdks/js/tests/validation.test.ts
describe('validation', () => {
  it('rejects empty email', () => { expect(() => validateEmail('')).toThrow('Email is required') })
  it('rejects invalid email', () => { expect(() => validateEmail('notanemail')).toThrow('Invalid email') })
  it('accepts valid email', () => { expect(() => validateEmail('a@b.com')).not.toThrow() })
  it('rejects short password', () => { expect(() => validatePassword('abc')).toThrow('at least 8') })
  it('accepts valid password', () => { expect(() => validatePassword('password123')).not.toThrow() })
})
```

**Step 4: Copy validation to React, Next.js SDKs**

**Step 5: Create Dart equivalent for Flutter**

```dart
// sdks/flutter/lib/src/validation.dart
class ValidationError implements Exception {
  final String field;
  final String message;
  ValidationError(this.field, this.message);
  @override String toString() => 'ValidationError($field): $message';
}

void validateEmail(String email) {
  if (email.trim().isEmpty) throw ValidationError('email', 'Email is required');
  if (!RegExp(r'^[^\s@]+@[^\s@]+\.[^\s@]+$').hasMatch(email)) throw ValidationError('email', 'Invalid email format');
}

void validatePassword(String password) {
  if (password.isEmpty) throw ValidationError('password', 'Password is required');
  if (password.length < 8) throw ValidationError('password', 'Password must be at least 8 characters');
}
```

**Step 6: Test all**

Run all SDK test suites.

**Step 7: Commit**

```bash
git commit -m "feat: add input validation for email and password across all SDKs"
```

---

## Verification Checklist

After all tasks complete:

```bash
# JS SDK
cd sdks/js && npx vitest run

# React SDK
cd sdks/react && npx vitest run

# Next.js SDK
cd sdks/nextjs && npx vitest run

# Node SDK
cd sdks/node && npx vitest run

# Flutter SDK
cd sdks/flutter && flutter test

# TypeScript check all
for sdk in js react nextjs node; do echo "=== $sdk ===" && cd sdks/$sdk && npx tsc --noEmit && cd ../..; done

# Dart analyze
cd sdks/flutter && dart analyze
```

All must pass before PR.

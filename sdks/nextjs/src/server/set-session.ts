'use server'
import { cookies } from 'next/headers'

export async function setNucleusSession(token: string, refreshToken: string, expiresAt: string) {
  const cookieStore = await cookies()
  const expires = new Date(expiresAt)
  const options = { httpOnly: true, secure: true, sameSite: 'lax' as const, path: '/', expires }
  cookieStore.set('__nucleus_session', token, options)
  cookieStore.set('__nucleus_refresh', refreshToken, options)
  cookieStore.set('__nucleus_expires', expiresAt, options)
}

export async function clearNucleusSession() {
  const cookieStore = await cookies()
  cookieStore.delete('__nucleus_session')
  cookieStore.delete('__nucleus_refresh')
  cookieStore.delete('__nucleus_expires')
}

'use server'
import { cookies } from 'next/headers'

export async function setNucleusSession(token: string, refreshToken: string, expiresAt: string) {
  const cookieStore = await cookies()
  const sessionExpires = new Date(expiresAt)
  const refreshExpires = new Date(Date.now() + 30 * 24 * 60 * 60 * 1000) // 30 days
  const base = { httpOnly: true, secure: true, sameSite: 'lax' as const, path: '/' }
  cookieStore.set('__nucleus_session', token, { ...base, expires: sessionExpires })
  cookieStore.set('__nucleus_refresh', refreshToken, { ...base, expires: refreshExpires })
  cookieStore.set('__nucleus_expires', expiresAt, { ...base, expires: refreshExpires })
}

export async function clearNucleusSession() {
  const cookieStore = await cookies()
  cookieStore.delete('__nucleus_session')
  cookieStore.delete('__nucleus_refresh')
  cookieStore.delete('__nucleus_expires')
}

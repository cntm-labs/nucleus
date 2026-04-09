import { describe, it, expect } from 'vitest'
import React from 'react'
import { render, screen } from '@testing-library/react'
import { I18nContext, useTranslation } from '../src/i18n'
import { en } from '../src/i18n/locales/en'
import { th } from '../src/i18n/locales/th'

function TestComponent({ translationKey, vars }: { translationKey: string; vars?: Record<string, string> }) {
  const t = useTranslation()
  return <span>{t(translationKey, vars)}</span>
}

describe('i18n', () => {
  it('defaults to English', () => {
    render(
      <I18nContext.Provider value={en}>
        <TestComponent translationKey="signIn.title" />
      </I18nContext.Provider>
    )
    expect(screen.getByText('Sign In')).toBeDefined()
  })

  it('uses Thai locale', () => {
    render(
      <I18nContext.Provider value={th}>
        <TestComponent translationKey="signIn.title" />
      </I18nContext.Provider>
    )
    expect(screen.getByText('เข้าสู่ระบบ')).toBeDefined()
  })

  it('interpolates variables', () => {
    render(
      <I18nContext.Provider value={en}>
        <TestComponent translationKey="signIn.oauth" vars={{ provider: 'Google' }} />
      </I18nContext.Provider>
    )
    expect(screen.getByText('Continue with Google')).toBeDefined()
  })

  it('returns key when translation missing', () => {
    render(
      <I18nContext.Provider value={en}>
        <TestComponent translationKey="nonexistent.key" />
      </I18nContext.Provider>
    )
    expect(screen.getByText('nonexistent.key')).toBeDefined()
  })

  it('interpolates variables in Thai', () => {
    render(
      <I18nContext.Provider value={th}>
        <TestComponent translationKey="signIn.oauth" vars={{ provider: 'Google' }} />
      </I18nContext.Provider>
    )
    expect(screen.getByText('ดำเนินการต่อด้วย Google')).toBeDefined()
  })
})

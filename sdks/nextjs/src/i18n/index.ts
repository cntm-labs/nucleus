import { createContext, useContext } from 'react'
import { en } from './locales/en'

export type Locale = Record<string, string>

export const I18nContext = createContext<Locale>(en)

export function useTranslation() {
  const locale = useContext(I18nContext)
  return (key: string, vars?: Record<string, string>) => {
    let text = locale[key] ?? key
    if (vars) {
      for (const [k, v] of Object.entries(vars)) {
        text = text.replace(`{{${k}}}`, v)
      }
    }
    return text
  }
}

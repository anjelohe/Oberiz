import { useSyncExternalStore } from 'react'
import { spanish } from './spanish'
import { european } from './european'

export const portalLanguages = ['en', 'es', 'fr', 'de', 'it', 'pt'] as const
export type PortalLanguage = typeof portalLanguages[number]
const catalogues: Record<PortalLanguage, Record<string, string>> = {
  en: {}, es: spanish, ...european,
}
const locales: Record<PortalLanguage, string> = {
  en: 'en-US', es: 'es-ES', fr: 'fr-FR', de: 'de-DE', it: 'it-IT', pt: 'pt-PT',
}
export function normalizeLanguage(value: unknown): PortalLanguage {
  return portalLanguages.includes(value as PortalLanguage) ? value as PortalLanguage : 'en'
}
let language = normalizeLanguage(localStorage.getItem('oberiz.language'))
document.documentElement.lang = language
const listeners = new Set<() => void>()
export function setPortalLanguage(value: PortalLanguage) {
  language = normalizeLanguage(value)
  localStorage.setItem('oberiz.language', language)
  document.documentElement.lang = language
  listeners.forEach(listener => listener())
}
export function useTranslation() {
  const current = useSyncExternalStore(listener => {
    listeners.add(listener)
    return () => { listeners.delete(listener) }
  }, () => language)
  return (text: string) => catalogues[current][text] ?? text
}
export function t(text: string): string {
  return catalogues[language][text] ?? text
}
export function portalLocale(): string {
  return locales[language]
}
export function tf(text: string, ...values: unknown[]): string {
  return t(text).replace(/\{(\d+)\}/g, (token, index: string) => Number(index) < values.length ? String(values[Number(index)]) : token)
}

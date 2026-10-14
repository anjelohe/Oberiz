const assert = require('node:assert/strict')
const fs = require('node:fs')
const vm = require('node:vm')
const ts = require('typescript')
const path = require('node:path')
const root = path.join(__dirname, '..')
function load(file, requireModule, globals = {}) {
  const js = ts.transpileModule(fs.readFileSync(path.join(root, file), 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText
  const exports = {}
  vm.runInNewContext(js, { exports, require: requireModule, ...globals })
  return exports
}
const catalogue = load('src/lib/spanish.ts', () => {})
const additional = load('src/lib/european.ts', name => {
  if (name === './spanish') return catalogue
  throw new Error(name)
})
const storage = new Map()
const doc = { documentElement: { lang: '' } }
const i18n = load('src/lib/i18n.ts', name => {
  if (name === './spanish') return catalogue
  if (name === './european') return additional
  if (name === 'react') return { useSyncExternalStore: (_, snapshot) => snapshot() }
  throw new Error(name)
}, { localStorage: { getItem: key => storage.get(key), setItem: (key, value) => storage.set(key, value) }, document: doc })
assert.equal(i18n.t('Settings'), 'Settings')
i18n.setPortalLanguage('es')
assert.equal(i18n.t('Settings'), 'Ajustes')
assert.equal(i18n.t('Allow upgrades'), 'Permitir mejoras')
assert.equal(i18n.t('Movies'), 'Películas')
assert.equal(i18n.tf('Delete {0}?', 'Movies {1}'), '¿Borrar Movies {1}?')
assert.equal(i18n.tf('Season {0}', 3), 'Temporada 3')
assert.equal(i18n.portalLocale(), 'es-ES')
assert.equal(storage.get('oberiz.language'), 'es')
assert.equal(doc.documentElement.lang, 'es')
assert.equal(i18n.t('My custom movie title'), 'My custom movie title')
assert.equal(i18n.t('https://tracker.example/download.php?id=12'), 'https://tracker.example/download.php?id=12')
i18n.setPortalLanguage('en')
assert.equal(i18n.t('Movies'), 'Movies')
assert.equal(i18n.tf('Season {0}', 3), 'Season 3')
assert.equal(i18n.portalLocale(), 'en-US')
assert.equal(doc.documentElement.lang, 'en')
for (const [english, spanish] of Object.entries(catalogue.spanish)) {
  assert.ok(spanish, 'Empty translation: ' + english)
  const sourceTokens = new Set(english.match(/\{\d+\}/g) || [])
  for (const token of spanish.match(/\{\d+\}/g) || []) assert.ok(sourceTokens.has(token), 'Unexpected placeholder: ' + english)
}
const examples = {
  fr: ['Paramètres', 'Saison 3', 'fr-FR'],
  de: ['Einstellungen', 'Staffel 3', 'de-DE'],
  it: ['Impostazioni', 'Stagione 3', 'it-IT'],
  pt: ['Definições', 'Temporada 3', 'pt-PT'],
}
for (const [code, [settings, season, locale]] of Object.entries(examples)) {
  i18n.setPortalLanguage(code)
  assert.equal(i18n.t('Settings'), settings)
  assert.equal(i18n.tf('Season {0}', 3), season)
  assert.equal(i18n.portalLocale(), locale)
  assert.equal(doc.documentElement.lang, code)
  assert.equal(storage.get('oberiz.language'), code)
  assert.equal(i18n.t('My custom movie title'), 'My custom movie title')
  assert.equal(i18n.t('https://tracker.example/download.php?id=12'), 'https://tracker.example/download.php?id=12')
  for (const english of Object.keys(catalogue.spanish)) {
    const translation = additional.european[code][english]
    assert.ok(translation, code + ' missing translation: ' + english)
    const tokens = new Set(english.match(/\{\d+\}/g) || [])
    for (const token of translation.match(/\{\d+\}/g) || []) assert.ok(tokens.has(token), code + ' unexpected placeholder: ' + english)
    // Preserve meaningful data arguments; grammatical plural suffix arguments may be omitted.
    for (const token of tokens) {
      if (!/issue\{1\}|episode\{1\}|record\{1\}|season\{1\}|feed\{1\}/.test(english) || token !== '{1}') {
        assert.ok(translation.includes(token), code + ' missing placeholder: ' + english)
      }
    }
  }
  i18n.setPortalLanguage('en')
  assert.equal(i18n.t('Settings'), 'Settings')
}
assert.equal(i18n.normalizeLanguage('unknown'), 'en')
assert.equal(i18n.normalizeLanguage(null), 'en')
assert.equal(i18n.normalizeLanguage('de'), 'de')
console.log('Six languages: catalogue coverage, placeholders, persistence, locales and switching OK')

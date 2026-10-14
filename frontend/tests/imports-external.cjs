const assert = require('node:assert/strict')
const fs = require('node:fs')
const path = require('node:path')
const vm = require('node:vm')
const ts = require('typescript')

async function scenario(mode, failure = false) {
  const values = []
  const effects = []
  let index = 0
  const jsx = (type, props) => ({ type, props })
  const row = { id: 1, status: 'seeding', release_title: 'Synthetic old import',
    media_type: 'movie', torrent_metadata_path: '/synthetic/old.torrent', library_path: '/synthetic/library' }
  const source = fs.readFileSync(path.join(__dirname, '../src/pages/Imports.tsx'), 'utf8')
  const output = ts.transpileModule(source, { compilerOptions: {
    module: ts.ModuleKind.CommonJS, jsx: ts.JsxEmit.ReactJSX,
  } }).outputText
  const context = { exports: {}, setInterval: () => 1, clearInterval: () => {}, require(name) {
    if (name === 'react') return {
      useState(initial) {
        const slot = index++
        if (!(slot in values)) values[slot] = initial
        return [values[slot], next => { values[slot] = typeof next === 'function' ? next(values[slot]) : next }]
      },
      useEffect: callback => effects.push(callback),
    }
    if (name === 'react/jsx-runtime') return { jsx, jsxs: jsx }
    if (name.endsWith('/i18n')) return { t: value => value, tf: value => value, useTranslation: () => {} }
    if (name.endsWith('/api')) return {
      getImports: async () => [row],
      getSettings: async () => {
        if (failure) throw new Error('Synthetic settings failure')
        return { import_method: mode }
      },
    }
    return {}
  } }
  vm.runInNewContext(output, context)
  context.exports.Imports()
  effects[0]()
  await new Promise(resolve => setImmediate(resolve))
  index = 0
  const tree = context.exports.Imports()
  function hasReseed(node) {
    if (!node || typeof node !== 'object') return false
    if (node.type === 'button' && node.props.children === 'Reseed') return true
    return [node.props?.children].flat(Infinity).some(hasReseed)
  }
  return hasReseed(tree)
}

;(async () => {
  assert.equal(await scenario('external'), false)
  assert.equal(await scenario('auto'), true)
  assert.equal(await scenario('auto', true), false)
  console.log('Old imports cannot offer reseed in external mode or when settings cannot be verified OK')
})().catch(error => { console.error(error); process.exitCode = 1 })

const assert = require('node:assert/strict')
const fs = require('node:fs')
const path = require('node:path')
const vm = require('node:vm')
const ts = require('typescript')
const mapping = { remote: '/media/Disk', local: '\\\\nas\\Disk', verified: true }
const hookValues = [
  [{ name: 'movies', save_path: '/media/Disk/movies' }, { name: '4k', save_path: '/media/Disk/movies4k' }],
  ['movies', '4k'], [mapping], true, '',
]
const jsx = (type, props) => ({ type, props })
const modules = {}
function load(relative) {
  if (modules[relative]) return modules[relative]
  const source = fs.readFileSync(path.join(__dirname, relative), 'utf8')
  const output = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.CommonJS, jsx: ts.JsxEmit.ReactJSX } }).outputText
  const context = { exports: {}, require(name) {
    if (name === 'react') return { useState: () => [hookValues.shift(), () => {}] }
    if (name === 'react/jsx-runtime') return { jsx, jsxs: jsx }
    if (name.endsWith('/i18n')) return { t: text => text, tf: text => text }
    if (name === '../components/StorageSettings') return load('../src/components/StorageSettings.tsx')
    return {}
  } }
  vm.runInNewContext(output + (relative.endsWith('/Settings.tsx') ? '\nexports.Editor = LibraryRootsEditor;' : ''), context)
  return modules[relative] = context.exports
}
const { Editor } = load('../src/pages/Settings.tsx')
const updates = []
const tree = Editor({ title: 'Movies', paths: ['existing'], mappings: [], onChange: (paths, mappings) => updates.push({ paths, mappings }) })
function find(node) {
  if (!node || typeof node !== 'object') return
  if (node.type === 'button' && node.props.children === 'Add selected folders') return node
  for (const child of [node.props?.children].flat(Infinity)) {
    const result = find(child)
    if (result) return result
  }
}
find(tree).props.onClick()
assert.equal(updates.length, 1, 'Folders and mappings must be delivered together')
assert.equal(JSON.stringify(updates[0].paths), JSON.stringify(['existing', '\\\\nas\\Disk\\movies', '\\\\nas\\Disk\\movies4k']))
assert.equal(updates[0].mappings.length, 1, 'Shared mappings must be deduplicated')
assert.equal(updates[0].mappings[0].remote, mapping.remote)
assert.equal(updates[0].mappings[0].local, mapping.local)
console.log('Category folders and new mappings are saved together OK')

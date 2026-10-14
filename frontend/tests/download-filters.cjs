const assert = require('node:assert/strict')
const fs = require('node:fs')
const path = require('node:path')
const vm = require('node:vm')
const ts = require('typescript')
const source = fs.readFileSync(path.join(__dirname, '../src/lib/downloadFilters.ts'), 'utf8')
const output = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.CommonJS } }).outputText
const context = { exports: {} }
vm.runInNewContext(output, context)
const matches = context.exports.matchesDownloadMetadata
const torrent = { category: 'Series', tags: 'Spanish, 1080p, Custom tag' }
const empty = { tracker: '', category: '', tag: '' }
assert.equal(matches(torrent, 'tracker.example', empty), true)
for (const filters of [{ tracker: 'wrong' }, { category: '"Movies"' }, { tag: 'Spanish2' }, { tag: 'Custom' }]) {
  assert.equal(matches(torrent, 'tracker.example', { ...empty, ...filters }), false)
}
assert.equal(matches(torrent, 'tracker.example', { tracker: 'tracker.example', category: '"Series"', tag: 'Custom tag' }), true)
assert.equal(matches({category:'',tags:''}, '', {...empty,category:'""'}),true)
assert.equal(matches({category:'__uncategorized__',tags:''}, '', {...empty,category:'""'}),false)
assert.equal(matches(torrent, 'tracker.example', {...empty,tag:'spanish'}),false)
console.log('Download metadata filters: intersection, exact tags, whitespace and empty categories OK')

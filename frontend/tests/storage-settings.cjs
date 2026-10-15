const assert = require('node:assert/strict')
const fs = require('node:fs')
const path = require('node:path')
const vm = require('node:vm')
const ts = require('typescript')
const source = fs.readFileSync(path.join(__dirname, '../src/components/StorageSettings.tsx'), 'utf8')
const output = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.CommonJS, jsx: ts.JsxEmit.ReactJSX } }).outputText
const jsx = (type,props) => ({type,props})
const context={exports:{},require(name){
  if(name==='react/jsx-runtime')return {jsx,jsxs:jsx}
  if(name.endsWith('/i18n'))return {t:text=>text,useTranslation:()=>{}}
  return {}
}}
vm.runInNewContext(output,context)
function find(node,predicate){
  if(!node||typeof node!=='object')return
  if(predicate(node))return node
  for(const child of [node.props?.children].flat(Infinity)){
    const result=find(child,predicate);if(result)return result
  }
}
for(const mode of ['local','cinetta']){
  const changes=[]
  const data={library_source:mode,rename_enabled:false,last_external_report:null}
  const tree=context.exports.StorageSettings({data,setData:next=>changes.push(next),existingLibrary:jsx('local-scanner',{})})
  assert.equal(Boolean(find(tree,node=>node.type==='local-scanner')),mode==='local')
  assert.equal(Boolean(find(tree,node=>node.type==='code'&&node.props.children==='PUT /api/v1/library')),mode==='cinetta')
  const options=[]
  find(tree,node=>{if(node.type==='input'&&node.props.type==='radio')options.push(node);return false})
  assert.equal(options.length,2)
  for(const option of options)option.props.onChange()
  assert.equal(changes[0].library_source,'cinetta')
  assert.equal(changes[1].library_source,'local')
  assert.ok(changes.every(next=>next.rename_enabled===false))
}
assert.ok(!source.includes('SmbConnections'))
assert.ok(!source.includes('getPathMappingSuggestions'))
console.log('Independent inventory source, read-only local scanner and no SMB/mapping configuration OK')

const assert=require('node:assert/strict')
const fs=require('node:fs')
const path=require('node:path')
const vm=require('node:vm')
const ts=require('typescript')
const crypto=require('node:crypto')
const source=fs.readFileSync(path.join(__dirname,'../src/components/MissingReports.tsx'),'utf8')
const settingsSource=fs.readFileSync(path.join(__dirname,'../src/pages/Settings.tsx'),'utf8')
assert.ok(!settingsSource.includes('cinetta_inventory_key'))
assert.ok(!settingsSource.includes('cinetta_url'))
assert.ok(settingsSource.includes('Reports use the last inventory Cinetta sent to Oberiz.'))
const code=ts.transpileModule(source,{compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022,jsx:ts.JsxEmit.ReactJSX}}).outputText
const values=[],calls=[],effects=[]
let cursor=0,closed=false
const jsx=(type,props)=>({type,props})
let result={state:'partial',job_id:'original_job',generated_at:'2026-10-16T00:00:00Z',inventory_generated_at:null,filters:{scope:'series_one',tmdb_id:202},warnings:['inventory_incomplete'],totals:{groups:1,episodes:600,movies:0},groups:[],page:1,next_page:2}
const context={exports:{},AbortController,URLSearchParams,crypto,setTimeout,clearTimeout,
  fetch:async(url,options)=>{calls.push({url,options});return{ok:true,json:async()=>result}},
  require(name){
    if(name==='react/jsx-runtime')return{jsx,jsxs:jsx}
    if(name==='react')return{
      useState(initial){const index=cursor++;if(!(index in values))values[index]=initial;return[values[index],value=>{values[index]=value}]},
      useRef(initial){const index=cursor++;if(!(index in values))values[index]={current:initial};return values[index]},
      useEffect(effect){effects.push(effect)},
    }
    if(name.endsWith('/i18n'))return{t:text=>text,useTranslation(){},portalLocale:()=> 'es-ES'}
    if(name==='./Icon')return{Icon:()=>null}
    throw new Error(name)
  },
}
vm.runInNewContext(code,context)
function render(){cursor=0;return context.exports.MissingReports({target:{scope:'series_one',tmdb_id:202},onClose:()=>{closed=true}})}
function find(node,predicate){if(!node||typeof node!=='object')return;if(predicate(node))return node;for(const child of [node.props?.children].flat(Infinity)){const match=find(child,predicate);if(match)return match}}
function button(tree,label){return find(tree,node=>node.type==='button'&&(node.props.children===label||find(node,child=>child.type==='span'&&child.props.children===label)))}
async function settle(){await new Promise(resolve=>setTimeout(resolve,10))}
;(async()=>{
  let tree=render();assert.equal(calls.length,0,'opening dialog must not fetch or start work')
  button(tree,'Update').props.onClick();await settle()
  assert.equal(calls[0].options.method,'POST')
  assert.match(calls[0].options.headers['Idempotency-Key'],/^[a-f0-9]{32}$/)
  assert.deepEqual(JSON.parse(calls[0].options.body),{scope:'series_one',tmdb_id:202})
  tree=render();assert.ok(find(tree,node=>node.type==='h3'&&node.props.children==='Provisional report'))
  result={...result,page:2,next_page:null}
  button(tree,'Next').props.onClick();await settle()
  const query=new URLSearchParams(calls[1].url.split('?')[1]);assert.equal(query.get('job_id'),'original_job');assert.equal(query.get('page'),'2');assert.equal(query.get('tmdb_id'),'202');assert.equal(calls[1].options.method,'GET')
  tree=render();button(tree,'Last saved report').props.onClick();await settle();assert.equal(calls[2].options.method,'GET');assert.ok(!calls[2].url.includes('refresh'))
  const before=values[2]
  context.fetch=async()=>({ok:false,text:async()=> 'cinetta_unavailable'})
  tree=render();button(tree,'Update').props.onClick();await settle();tree=render()
  assert.ok(find(tree,node=>node.props.role==='alert'));assert.equal(values[2],before,'errors preserve previous report')
  assert.ok(!calls.some(call=>/requests|downloads|rescan/.test(call.url)))
  find(tree,node=>node.props['aria-label']==='Close').props.onClick();assert.equal(closed,true)
  console.log('Missing reports: explicit actions, read-only requests, immutable pagination and previous report preserved OK')
})().catch(error=>{console.error(error);process.exitCode=1})

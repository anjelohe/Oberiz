import { useCallback, useEffect, useMemo, useState } from 'react'
import { Icon } from '../components/Icon'
import './Indexers.css'

type Indexer={
  id:string;name:string;description:string|null;language:string|null;indexer_type:string|null;
  links:string[];source:string;file_name:string;valid:boolean;configured:boolean;enabled:boolean;
  priority:number;
  settings_count:number;last_status:'unknown'|'online'|'offline'|string;last_message:string|null;
  last_latency_ms:number|null;last_checked_at:string|null;error:string|null
}
type ListResponse={status:string;total:number;upstream:number;custom:number;enabled:number;invalid:number;indexers:Indexer[]}
type Setting={name:string;label:string;field_type:string;secret:boolean;configured:boolean;value:any;default:any;options:[string,string][]}
type Detail={id:string;name:string;description:string|null;indexer_type:string|null;language:string|null;links:string[];enabled:boolean;settings:Setting[]}

function relative(value:string|null){
  if(!value)return 'Never'
  const time=new Date(value.endsWith('Z')?value:`${value}Z`).getTime()
  if(!Number.isFinite(time))return value
  const s=Math.max(0,Math.floor((Date.now()-time)/1000))
  if(s<60)return 'just now'
  if(s<3600)return `${Math.floor(s/60)} min ago`
  if(s<86400)return `${Math.floor(s/3600)} h ago`
  return `${Math.floor(s/86400)} d ago`
}
function typeLabel(type:string|null){return type==='private'?'Private':'Public'}
function categoryGuess(item:Indexer){
  const text=`${item.name} ${item.description??''}`.toLowerCase()
  const out:string[]=[]
  if(/movie|film|cinema/.test(text))out.push('Movies')
  if(/tv|series|anime/.test(text))out.push('TV')
  if(/music|audio/.test(text))out.push('Audio')
  if(/game/.test(text))out.push('Games')
  if(/xxx|adult/.test(text))out.push('XXX')
  return out.length?out.slice(0,3):['General']
}

export function Indexers(){
  const [data,setData]=useState<ListResponse|null>(null)
  const [query,setQuery]=useState('')
  const [catalogueQuery,setCatalogueQuery]=useState('')
  const [catalogueOpen,setCatalogueOpen]=useState(false)
  const [busy,setBusy]=useState(false),[error,setError]=useState('')
  const [selected,setSelected]=useState<Detail|null>(null),[values,setValues]=useState<Record<string,any>>({}),[test,setTest]=useState('')

  const load=useCallback(async()=>{const r=await fetch('/api/indexers');if(!r.ok)throw new Error(await r.text());setData(await r.json())},[])
  useEffect(()=>{void load().catch(e=>setError(String(e)))},[load])

  async function sync(){setBusy(true);setError('');try{const r=await fetch('/api/indexers/sync',{method:'POST'});if(!r.ok)throw new Error(await r.text());await load()}catch(e){setError(e instanceof Error?e.message:String(e))}finally{setBusy(false)}}
  async function open(item:Indexer){setTest('');const r=await fetch(`/api/indexers/${encodeURIComponent(item.id)}`);if(!r.ok){setError(await r.text());return}const d:Detail=await r.json();setSelected(d);const initial:Record<string,any>={};d.settings.forEach(s=>initial[s.name]=s.value??s.default??(s.field_type==='checkbox'?false:''));setValues(initial)}
  async function save(closeAfterSaving=true):Promise<boolean>{
    if(!selected)return false;setBusy(true);setTest('')
    try{
      let r=await fetch(`/api/indexers/${encodeURIComponent(selected.id)}/config`,{method:'PUT',headers:{'Content-Type':'application/json'},body:JSON.stringify({values})});if(!r.ok)throw new Error(await r.text())
      r=await fetch(`/api/indexers/${encodeURIComponent(selected.id)}/enabled`,{method:'PUT',headers:{'Content-Type':'application/json'},body:JSON.stringify({enabled:selected.enabled})});if(!r.ok)throw new Error(await r.text())
      await load()
      if(closeAfterSaving)setSelected(null)
      else setTest('Saved')
      return true
    }catch(e){setTest(e instanceof Error?e.message:String(e));return false}finally{setBusy(false)}
  }
  async function testIndexer(){
    if(!selected)return;setBusy(true);setTest('Testing…')
    try{if(!await save(false))return;const r=await fetch(`/api/indexers/${encodeURIComponent(selected.id)}/test`,{method:'POST'});if(!r.ok)throw new Error(await r.text());const x=await r.json();setTest(x.message);await load()}
    catch(e){setTest(e instanceof Error?e.message:String(e));await load()}
    finally{setBusy(false)}
  }
  async function toggle(item:Indexer){
    const r=await fetch(`/api/indexers/${encodeURIComponent(item.id)}/enabled`,{method:'PUT',headers:{'Content-Type':'application/json'},body:JSON.stringify({enabled:!item.enabled})})
    if(!r.ok){setError(await r.text());return}
    await load()
  }

  async function removeConfigured(item:Indexer){
    if(!confirm(`Remove ${item.name} from configured indexers?\n\nThe definition will remain available in Add Indexer. Seed policies and download history are preserved.`))return
    const r=await fetch(`/api/indexers/${encodeURIComponent(item.id)}`,{method:'DELETE'})
    if(!r.ok){setError(await r.text());return}
    await load()
  }

  const configured=useMemo(()=>{
    const q=query.trim().toLowerCase()
    return (data?.indexers??[]).filter(x=>x.configured).filter(x=>!q||[x.name,x.id,x.language??'',x.indexer_type??'',x.last_status].some(v=>v.toLowerCase().includes(q)))
  },[data,query])
  const catalogue=useMemo(()=>{
    const q=catalogueQuery.trim().toLowerCase()
    return (data?.indexers??[]).filter(x=>!x.configured).filter(x=>!q||[x.name,x.id,x.language??'',x.description??'',x.indexer_type??''].some(v=>v.toLowerCase().includes(q)))
  },[data,catalogueQuery])

  const enabled=configured.filter(x=>x.enabled)
  const online=enabled.filter(x=>x.last_status==='online').length
  const needsTest=enabled.filter(x=>x.last_status==='unknown').length

  return <div className="ix-page">
    <div className="page-heading ix-page-heading">
      <div><h1><Icon name="indexers" size={32}/> Indexers</h1><p>Configured indexers first. Definitions stay in the Add Indexer catalogue.</p></div>
      <div className="ix-heading-actions">
        <button className="ghost-button" onClick={()=>void sync()} disabled={busy}>{busy?'Syncing…':'Reload Definitions'}</button>
        <button className="primary-button" onClick={()=>setCatalogueOpen(true)}><Icon name="plus" size={18}/> Add Indexer</button>
      </div>
    </div>

    {error&&<div className="error-box">{error}</div>}

    <div className="ix-overview-grid">
      <div className="ix-overview-card card"><div className="ix-overview-icon good"><Icon name="indexers"/></div><div><span>Configured Indexers</span><strong>{configured.length}</strong><small>Your configured indexers</small></div></div>
      <div className="ix-overview-card card"><div className="ix-overview-icon defs"><Icon name="check"/></div><div><span>Enabled</span><strong>{enabled.length}</strong><small>{enabled.length} of {configured.length} enabled</small></div></div>
      <div className="ix-overview-card card"><div className="ix-overview-icon sync"><Icon name="history"/></div><div><span>Online</span><strong>{online}</strong><small>{online} of {enabled.length} enabled online</small></div></div>
      <div className="ix-overview-card card"><div className="ix-overview-icon warning"><Icon name="history"/></div><div><span>Needs Test</span><strong>{needsTest}</strong><small>{needsTest} enabled not tested yet</small></div></div>
    </div>

    <section className="card ix-installed-panel">
      <div className="ix-toolbar">
        <div><strong>Indexers ({configured.length})</strong><span>Only configured indexers appear here.</span></div>
        <div className="ix-search"><Icon name="search" size={16}/><input value={query} onChange={e=>setQuery(e.target.value)} placeholder="Search configured indexers..."/></div>
      </div>

      <div className="ix-table-head"><span>Name</span><span>Type</span><span>Status</span><span>Categories</span><span>Priority</span><span>Last Check</span><span>Latency</span><span>Actions</span></div>
      <div className="ix-installed-list">
        {configured.map(item=><div className="ix-installed-row" key={`${item.source}-${item.id}`}>
          <button className="ix-name-button" onClick={()=>void open(item)}>
            <span className="ix-logo">{item.name.slice(0,1).toUpperCase()}</span>
            <span><strong>{item.name}</strong><small>{item.language||'—'} · {item.source}</small></span>
          </button>
          <span><b className={`ix-type ${item.indexer_type==='private'?'private':'public'}`}>{typeLabel(item.indexer_type)}</b></span>
          <span><b className={`ix-runtime ${item.last_status}`}>{item.last_status==='online'?'● Online':item.last_status==='offline'?'● Offline':'● Not tested'}</b></span>
          <span className="ix-categories">{categoryGuess(item).map(c=><em key={c}>{c}</em>)}</span>
          <span>{item.priority}</span>
          <span>{relative(item.last_checked_at)}</span>
          <span>{item.last_latency_ms!=null?`${item.last_latency_ms} ms`:'—'}</span>
          <span className="ix-row-actions">
            <button className={`ix-switch ${item.enabled?'on':''}`} onClick={()=>void toggle(item)} aria-label={item.enabled?'Disable':'Enable'}><i/></button>
            <button className="ix-icon-button" onClick={()=>void open(item)} title="Edit">✎</button>
            <button className="ix-icon-button ix-delete-button" onClick={()=>void removeConfigured(item)} title="Remove from configured indexers">⌫</button>
          </span>
        </div>)}
        {!configured.length&&<div className="empty-state">No indexers configured yet. Use <strong>Add Indexer</strong> to choose one from the definition catalogue.</div>}
      </div>
    </section>

    <section className="ix-bottom-grid">
      <div className="card ix-info-card"><div className="ix-info-head"><strong>Definition Sources</strong><button className="ghost-button" onClick={()=>void sync()}>Reload</button></div>
        <div className="ix-source-row"><span>Prowlarr upstream</span><b>Loaded</b><em>{data?.upstream??0}</em></div>
        <div className="ix-source-row"><span>Custom definitions</span><b>Loaded</b><em>{data?.custom??0}</em></div>
      </div>
      <div className="card ix-info-card"><div className="ix-info-head"><strong>Indexer Activity</strong></div>
        {configured.slice(0,4).map(x=><div className="ix-source-row" key={x.id}><span>{x.name}</span><b className={x.last_status==='offline'?'danger-text':''}>{x.last_status==='unknown'?'Not tested':x.last_status}</b><em>{relative(x.last_checked_at)}</em></div>)}
        {!configured.length&&<div className="empty-state">Runtime tests and failures will appear here.</div>}
      </div>
    </section>

    {catalogueOpen&&<div className="ix-modal-bg" onMouseDown={()=>setCatalogueOpen(false)}>
      <div className="ix-catalogue-modal" onMouseDown={e=>e.stopPropagation()}>
        <div className="ix-modal-head"><div><p>ADD INDEXER</p><h2>Definition catalogue</h2><span>{catalogue.length} definitions available to configure</span></div><button onClick={()=>setCatalogueOpen(false)}>×</button></div>
        <div className="ix-catalogue-search"><Icon name="search" size={18}/><input value={catalogueQuery} onChange={e=>setCatalogueQuery(e.target.value)} placeholder="Search Prowlarr and custom definitions..."/></div>
        <div className="ix-catalogue-grid">
          {catalogue.slice(0,160).map(item=><button key={`${item.source}-${item.id}`} className="ix-catalogue-card" onClick={()=>{setCatalogueOpen(false);void open(item)}}>
            <span className="ix-logo">{item.name.slice(0,1).toUpperCase()}</span>
            <span className="ix-catalogue-copy"><strong>{item.name}</strong><small>{typeLabel(item.indexer_type)} · {item.language||'—'} · {item.source}</small><p>{item.description||item.links[0]||item.file_name}</p></span>
            <b>＋</b>
          </button>)}
        </div>
      </div>
    </div>}

    {selected&&<div className="ix-modal-bg" onMouseDown={()=>setSelected(null)}><div className="ix-modal" onMouseDown={e=>e.stopPropagation()}>
      <div className="ix-modal-head"><div><p>INDEXER SETTINGS</p><h2>{selected.name}</h2><span>{selected.indexer_type} · {selected.language} · {selected.links[0]}</span></div><button onClick={()=>setSelected(null)}>×</button></div>
      <label className="ix-enable"><input type="checkbox" checked={selected.enabled} onChange={e=>setSelected({...selected,enabled:e.target.checked})}/><span><strong>Enabled</strong><small>Use this indexer in release searches.</small></span></label>
      <div className="ix-fields">{selected.settings.map(field=><label key={field.name}>{field.label}<small>{field.secret&&field.configured?' · currently configured':''}</small>
        {field.field_type==='checkbox'?<input type="checkbox" checked={Boolean(values[field.name])} onChange={e=>setValues({...values,[field.name]:e.target.checked})}/>:
         field.field_type==='select'?<select value={String(values[field.name]??'')} onChange={e=>setValues({...values,[field.name]:e.target.value})}>{field.options.map(([v,l])=><option key={v} value={v}>{l}</option>)}</select>:
         <input type={field.secret?'password':field.field_type==='number'?'number':'text'} min={field.field_type==='number'?0:undefined} max={field.field_type==='number'?10000:undefined} step={field.field_type==='number'?1:undefined} value={String(values[field.name]??'')} onChange={e=>setValues({...values,[field.name]:e.target.value})} placeholder={field.name==='oberiz_tag_name'?'For example: ThePirateBay':field.name==='oberiz_priority'?'Lower number is preferred':field.secret&&field.configured?'Leave blank to keep current':''}/>}
      </label>)}</div>
      <small className="ix-routing-help">Lower priority runs first and breaks ties between equally scored releases. The custom tag replaces <b>[tracker]</b> in the selected quality profile’s tag template.</small>
      <div className="ix-test-result">{test}</div>
      <div className="ix-modal-actions"><button className="ghost-button" onClick={()=>void testIndexer()} disabled={busy}>Test</button><button className="primary-button" onClick={()=>void save()} disabled={busy}>Save</button></div>
    </div></div>}
  </div>
}

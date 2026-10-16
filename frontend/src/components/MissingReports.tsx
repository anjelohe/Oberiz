import { useEffect, useRef, useState } from 'react'
import { t, useTranslation, portalLocale } from '../lib/i18n'
import { Icon } from './Icon'

type Filters={scope:string;tmdb_id?:number;tvdb_id?:number}
type Item={title:string;season_number:number|null;episode_number:number|null;release_date:string;availability:string}
type Report={state:string;job_id:string;generated_at:string;inventory_generated_at:string|null;filters:Filters;warnings:string[];totals:{groups:number|null;episodes:number|null;movies:number|null};groups:Array<{type:string;tmdb_id:number;title:string;items:Item[]}>;page:number;next_page:number|null}
export type MissingTarget={scope:'series'|'collections'|'series_one'|'collection_one';tmdb_id?:number;title?:string}
const messages:Record<string,string>={
  cinetta_inventory_not_received:'No Cinetta inventory received yet. Enable inventory sending in Cinetta using the existing Oberiz API key.',
  tmdb_not_configured:'TMDB is not configured.',
  report_not_found:'No saved report. Use Update to start a check.',
  cinetta_access_denied:'Cinetta rejected the inventory key.',
  cinetta_inventory_unsupported:'Cinetta does not support inventory queries.',
  cinetta_unavailable:'Cinetta is unavailable.',
  movie_has_no_collection:'This movie does not belong to a TMDB collection.',
  invalid_filter:'Choose a valid series or collection ID.',
  too_many_checks:'Other checks are running. Try again later.',
}
export async function collectionForMovie(id:number):Promise<MissingTarget>{
  const response=await fetch(`/api/library/missing/collection?movie_tmdb_id=${id}`)
  if(!response.ok){const code=await response.text();throw new Error(t(messages[code]||'Could not check missing content.'))}
  const value=await response.json()
  return {scope:'collection_one',tmdb_id:value.tmdb_id,title:value.title}
}
export function MissingReports({target,onClose}:{target:MissingTarget;onClose:()=>void}){
  useTranslation()
  const [scope,setScope]=useState<string>(target.scope)
  const [externalId,setExternalId]=useState(target.tmdb_id?String(target.tmdb_id):'')
  const [report,setReport]=useState<Report|null>(null)
  const [busy,setBusy]=useState(false),[error,setError]=useState(''),[running,setRunning]=useState(false)
  const active=useRef<AbortController|null>(null)
  useEffect(()=>()=>active.current?.abort(),[])
  const cancel=()=>{active.current?.abort();active.current=null;setBusy(false);setRunning(false)}
  const close=()=>{cancel();onClose()}
  async function run(refresh:boolean,page?:number){
    if(active.current)return
    const controller=new AbortController();active.current=controller
    setBusy(true);setError('');setRunning(false)
    const filters:Filters & {page?:number;job_id?:string}=page&&report
      ? {...report.filters,page,job_id:report.job_id}
      : {scope,...(scope.endsWith('_one')?{tmdb_id:Number(externalId)}:{})}
    // getRandomValues also works on LAN HTTP origins, unlike randomUUID.
    const actionKey=Array.from(crypto.getRandomValues(new Uint8Array(16)),byte=>byte.toString(16).padStart(2,'0')).join('')
    try{
      for(let attempt=0;attempt<400;attempt++){
        const post=refresh&&attempt===0
        const response=await fetch(`/api/library/missing${post?'/refresh':`?${new URLSearchParams(Object.entries(filters).map(([key,value])=>[key,String(value)]))}`}`,{
          method:post?'POST':'GET',signal:controller.signal,
          headers:post?{'Content-Type':'application/json','Idempotency-Key':actionKey}:{},
          body:post?JSON.stringify(filters):undefined,
        })
        if(!response.ok){const code=await response.text();throw new Error(t(messages[code]||'Could not check missing content.'))}
        const value:Report=await response.json()
        if(controller.signal.aborted)return
        if(['queued','running'].includes(value.state)){
          if(!value.job_id)throw new Error(t('Could not check missing content.'))
          filters.job_id=value.job_id;setRunning(true)
          if(attempt===399)throw new Error(t('The check is still running. Consult the saved report later.'))
          await new Promise<void>(resolve=>{const finish=()=>{clearTimeout(timer);controller.signal.removeEventListener('abort',finish);resolve()};const timer=setTimeout(finish,3000);controller.signal.addEventListener('abort',finish,{once:true})})
          if(controller.signal.aborted)return
          continue
        }
        if(value.state==='error')throw new Error(t(messages[value.warnings[0]]||'The check failed. The previous report is unchanged.'))
        setReport(value);setRunning(false);return
      }
    }catch(e){if(!controller.signal.aborted)setError(e instanceof Error?e.message:String(e))}
    finally{if(active.current===controller){active.current=null;setBusy(false);setRunning(false)}}
  }
  const date=(value:string|null)=>value?new Date(value).toLocaleString(portalLocale()):'—'
  return <div className="dl-modal-backdrop" onMouseDown={close}><section className="dl-modal missing-report-modal" role="dialog" aria-modal="true" aria-labelledby="missing-heading" onMouseDown={event=>event.stopPropagation()}>
    <div className="dl-modal-heading missing-heading"><div className="missing-heading-title"><span className="missing-heading-icon"><Icon name="imports" size={22}/></span><h2 id="missing-heading">{t('Check missing content')}{target.title?` · ${target.title}`:''}</h2></div><button type="button" className="missing-close" aria-label={t('Close')} onClick={close}>×</button></div>
    <p className="settings-help">{t('Read-only report. No requests, monitoring or downloads are created.')}</p>
    <div className="missing-controls">
      <label>{t('Scope')}<select value={scope} disabled={busy} onChange={e=>{setScope(e.target.value);setReport(null);setError('')}}>
        <option value="series">{t('Missing episodes')}</option><option value="collections">{t('Incomplete collections')}</option><option value="series_one">{t('One series')}</option><option value="collection_one">{t('One collection')}</option><option value="all">{t('Entire library')}</option>
      </select></label>
      {scope.endsWith('_one')&&<label>{t('TMDB ID')}<input type="number" min="1" value={externalId} disabled={busy} onChange={e=>{setExternalId(e.target.value);setReport(null)}}/></label>}
      <button type="button" className="missing-secondary" disabled={busy} onClick={()=>void run(false)}><Icon name="history" size={16}/><span>{t('Last saved report')}</span></button>
      <button type="button" className="missing-primary" disabled={busy||scope.endsWith('_one')&&!/^[1-9]\d*$/.test(externalId)} onClick={()=>void run(true)}><Icon name="search" size={16}/><span>{t('Update')}</span></button>
      {busy&&<button type="button" className="missing-secondary" onClick={cancel}>{t('Stop waiting')}</button>}
    </div>
    {scope==='all'&&<p className="settings-help">{t('Updating will check the entire inventory and may take several minutes.')}</p>}
    {running&&<p role="status" className="missing-progress"><span aria-hidden="true" className="missing-spinner"/>{t('Checking inventory and release metadata…')}</p>}
    {error&&<p role="alert" className="error-box">{error}</p>}
    {report&&<div className="missing-results">
      <h3>{report.state==='partial'?t('Provisional report'):report.state==='no_matches'?t('No missing content found'):t('Report complete')}</h3>
      <p className="settings-help">{t('Report date')}: {date(report.generated_at)} · {t('Inventory date')}: {date(report.inventory_generated_at)}</p>
      {report.state==='partial'&&<p className="settings-help">{t('Not listed in Cinetta. Missing files are not physically confirmed; offline indexed items are not marked missing.')}</p>}
      <p>{t('Groups')}: {report.totals.groups??'—'} · {t('Episodes')}: {report.totals.episodes??'—'} · {t('Movies')}: {report.totals.movies??'—'}</p>
      {report.warnings.length>0&&<details><summary>{t('Warnings')}</summary><ul>{report.warnings.map(warning=><li key={warning}>{warning}</li>)}</ul></details>}
      {report.groups.map((group,index)=><details key={`${group.type}-${group.tmdb_id}-${index}`} className="missing-group" open={report.groups.length===1}><summary>{group.title} · {group.items.length}</summary><ul>{group.items.map((item,i)=><li key={i}>{item.season_number!==null?`S${item.season_number} E${item.episode_number} · `:''}{item.title} <small>{date(item.release_date)}</small></li>)}</ul></details>)}
      <div className="missing-controls"><span>{t('Page')} {report.page}</span>{report.page>1&&<button type="button" disabled={busy} onClick={()=>void run(false,report.page-1)}>{t('Previous')}</button>}{report.next_page&&<button type="button" disabled={busy} onClick={()=>void run(false,report.next_page!)}>{t('Next')}</button>}</div>
    </div>}
  </section></div>
}

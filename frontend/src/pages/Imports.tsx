import { t, useTranslation, tf } from '../lib/i18n'
import { useEffect, useState } from 'react'
import { deleteImports, getImports, ImportJob, reseedImport } from '../lib/api'
import { Icon } from '../components/Icon'

function pretty(status:string){return t(status.replaceAll('_',' ').replace(/\b\w/g,c=>c.toUpperCase()))}

export function Imports(){
  useTranslation()
  const [rows,setRows]=useState<ImportJob[]>([])
  const [message,setMessage]=useState('')
  const [busy,setBusy]=useState<number|null>(null)
  const [selected,setSelected]=useState<number[]>([])
  const [deleting,setDeleting]=useState(false)
  const [selectionAnchor,setSelectionAnchor]=useState<number|null>(null)

  async function load(){try{const next=await getImports();setRows(next);setSelected(current=>current.filter(id=>next.some(row=>row.id===id)))}catch(e){setMessage(e instanceof Error?e.message:String(e))}}
  useEffect(()=>{void load();const t=setInterval(()=>void load(),5000);return()=>clearInterval(t)},[])

  async function reseed(row:ImportJob){
    try{setBusy(row.id);const r=await reseedImport(row.id);setMessage(tf("Reseed started in {0}", r.path));await load()}
    catch(e){setMessage(e instanceof Error?e.message:String(e))}
    finally{setBusy(null)}
  }
  const removable=(row:ImportJob)=>['duplicate','error','missing','cleaned','rejected'].includes(row.status)
  const removableRows=rows.filter(removable)
  function toggle(id:number,range:boolean){
    if(range&&selectionAnchor!=null){
      const start=rows.findIndex(row=>row.id===selectionAnchor),end=rows.findIndex(row=>row.id===id)
      if(start>=0&&end>=0){
        const rangeIds=rows.slice(Math.min(start,end),Math.max(start,end)+1).filter(removable).map(row=>row.id)
        setSelected(current=>[...new Set([...current,...rangeIds])])
        return
      }
    }
    setSelected(current=>current.includes(id)?current.filter(value=>value!==id):[...current,id])
    setSelectionAnchor(id)
  }
  async function remove(ids:number[]){
    if(!ids.length)return
    if(!confirm(tf("Remove {0} obsolete Oberiz record{1}? qBittorrent torrents and media files will not be changed.", ids.length, ids.length===1?'':'s')))return
    try{setDeleting(true);const result=await deleteImports(ids);setMessage(tf("{0} record{1} removed.", result.deleted, result.deleted===1?'':'s'));await load()}
    catch(e){setMessage(e instanceof Error?e.message:String(e))}
    finally{setDeleting(false)}
  }

  return <>
    <div className="page-heading"><div><h1><Icon name="imports" size={32}/> {t("Imports & Reseed")}</h1><p>{t("Completed imports, seed cleanup and recoverable torrent metadata.")}</p></div></div>
    {message&&<div className="card import-message">{message}</div>}
    <section className="card imports-table">
      <div className="imports-toolbar"><span>{removableRows.length} {t("obsolete record")}{removableRows.length===1?'':'s'} {t("can be removed safely. Use Shift to select a range.")}</span><button className="danger-button" disabled={!selected.length||deleting} onClick={()=>void remove(selected)}>{deleting?t("Removing…"):tf("Remove selected{0}", selected.length?` (${selected.length})`:'')}</button></div>
      <div className="import-row import-head"><span>{t("Select")}</span><span>{t("Media")}</span><span>{t("Status")}</span><span>{t("Indexer")}</span><span>{t("Method")}</span><span>{t("Library")}</span><span>Reseed</span></div>
      {rows.length===0?<div className="empty-state">{t("No Oberiz import jobs yet.")}</div>:rows.map(row=>
        <div className="import-row" key={row.id}>
          <span><input type="checkbox" checked={selected.includes(row.id)} disabled={!removable(row)} onChange={event=>toggle(row.id,(event.nativeEvent as MouseEvent).shiftKey)} title={removable(row)?t("Select for removal — hold Shift to select a range"):t("Active records cannot be removed")}/></span>
          <div><strong>{row.release_title}</strong><small>{row.media_type}{row.season_number!=null?` · S${String(row.season_number).padStart(2,'0')}${row.episode_number!=null?`E${String(row.episode_number).padStart(2,'0')}`:row.is_season_pack?' pack':''}`:''} {t("· job #")}{row.id}</small></div>
          <span className={`badge ${row.status==='error'?'badge-red':row.status==='cleaned'?'badge-green':'badge-blue'}`}>{pretty(row.status)}</span>
          <span>{row.indexer_name||row.indexer_id}</span>
          <span>{row.import_method||'—'}</span>
          <span className="import-path">{row.library_path||row.last_error||'—'}</span>
          <div className="import-actions">{row.torrent_metadata_path&&row.library_path?<button className="ghost-button" disabled={busy===row.id} onClick={()=>void reseed(row)}>{busy===row.id?t("Starting…"):'Reseed'}</button>:<span title={t("This record has no reusable .torrent metadata and library path.")}>{t("Unavailable")}</span>}{removable(row)&&<button className="danger-button" disabled={deleting} onClick={()=>void remove([row.id])}>{t("Remove")}</button>}</div>
        </div>
      )}
    </section>
  </>
}

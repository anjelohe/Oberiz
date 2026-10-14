import { t, useTranslation, tf } from '../lib/i18n'
import { useEffect, useState } from 'react'
import { DirectoryListing, getDirectories } from '../lib/api'
import { useModalA11y } from '../lib/useModalA11y'

export function FolderPicker({value,onChange,label=t("Browse folders"),onSelectMany}:{value:string;onChange:(path:string)=>void;label?:string;onSelectMany?:(paths:string[])=>void}){
  useTranslation()
  const [open,setOpen]=useState(false),[listing,setListing]=useState<DirectoryListing|null>(null),[error,setError]=useState('')
  const [selected,setSelected]=useState<string[]>([])
  const [pathInput,setPathInput]=useState('')
  const multiple=Boolean(onSelectMany)
  function toggle(path:string){setSelected(paths=>paths.includes(path)?paths.filter(p=>p!==path):[...paths,path])}
  async function load(path?:string){try{setError('');const next=await getDirectories(path);setListing(next);setPathInput(next.path)}catch(e){setError(e instanceof Error?e.message:String(e))}}
  useEffect(()=>{if(open)void load(value||undefined)},[open])
  const dialogRef=useModalA11y<HTMLElement>(open,()=>setOpen(false))
  return <>
    <button type="button" className={multiple?'ghost-button':'folder-browse'} onClick={()=>{setSelected([]);setListing(null);setOpen(true)}}>{label}</button>
    {open&&<div className="folder-picker-backdrop" onMouseDown={()=>setOpen(false)}><section ref={dialogRef} className="folder-picker" role="dialog" aria-modal="true" aria-labelledby="folder-picker-title" tabIndex={-1} onMouseDown={event=>event.stopPropagation()}>
      <header><div><strong id="folder-picker-title">{t("Select folder")}</strong><small>{listing?(listing.path||t('All storage locations')):t("Loading…")}</small></div><button type="button" onClick={()=>setOpen(false)} aria-label={t("Close")}>×</button></header>
      <div className="folder-picker-actions"><button type="button" disabled={!listing?.parent} onClick={()=>void load(listing?.parent||undefined)}>{t("↑ Parent folder")}</button><button type="button" onClick={()=>void load(listing?.path||value||undefined)}>{t("↻ Refresh")}</button>{listing?.roots.map(root=><button type="button" className={root.toLowerCase()===listing.path.toLowerCase()?'active':''} key={root} onClick={()=>void load(root)}>{root}</button>)}</div>
      <div className="folder-picker-actions"><button type="button" onClick={()=>void load()}>{t('All storage locations')}</button><input aria-label={t('Folder path')} value={pathInput} onChange={event=>setPathInput(event.target.value)} onKeyDown={event=>{if(event.key==='Enter'){event.preventDefault();void load(pathInput)}}} placeholder="\\\\server\\share"/><button type="button" onClick={()=>void load(pathInput)}>{t('Open')}</button></div>
      {error?<p className="folder-picker-error">{error}</p>:<div className="folder-picker-list">
        {multiple&&listing?.path&&<label className="folder-picker-choice"><input type="checkbox" checked={selected.includes(listing.path)} onChange={()=>toggle(listing.path)}/><span>{t("Select current folder")}</span></label>}
        {listing?.directories.map(entry=>multiple?<div className="folder-picker-multi-row" key={entry.path}><input type="checkbox" aria-label={entry.name} checked={selected.includes(entry.path)} onChange={()=>toggle(entry.path)}/><button type="button" onClick={()=>void load(entry.path)}>▸ {entry.name}</button></div>:<button type="button" key={entry.path} onClick={()=>void load(entry.path)}>▸ {entry.name}</button>)}
        {listing&&!listing.directories.length&&<p>{t("No subfolders.")}</p>}
        {multiple&&selected.length>0&&<div className="folder-picker-selection">{selected.map(path=><label className="folder-picker-choice" key={path}><input type="checkbox" checked onChange={()=>toggle(path)}/><code>{path}</code></label>)}</div>}
      </div>}
      <footer><code>{multiple?tf("{0} folders selected",selected.length):listing?.path||value}</code><button type="button" className="primary-button" disabled={multiple?!selected.length:!listing?.path} onClick={()=>{if(onSelectMany)onSelectMany(selected);else if(listing)onChange(listing.path);setOpen(false)}}>{multiple?t("Add selected folders"):t("Use this folder")}</button></footer>
    </section></div>}
  </>
}

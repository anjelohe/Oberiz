import { useEffect, useState } from 'react'
import { DirectoryListing, getDirectories } from '../lib/api'

export function FolderPicker({value,onChange,label='Browse folders'}:{value:string;onChange:(path:string)=>void;label?:string}){
  const [open,setOpen]=useState(false),[listing,setListing]=useState<DirectoryListing|null>(null),[error,setError]=useState('')
  async function load(path?:string){try{setError('');setListing(await getDirectories(path))}catch(e){setError(e instanceof Error?e.message:String(e))}}
  useEffect(()=>{if(open)void load(value||undefined)},[open])
  return <>
    <button type="button" className="folder-browse" onClick={()=>setOpen(true)}>{label}</button>
    {open&&<div className="folder-picker-backdrop" onMouseDown={()=>setOpen(false)}><section className="folder-picker" onMouseDown={event=>event.stopPropagation()}>
      <header><div><strong>Select folder</strong><small>{listing?.path||'Loading…'}</small></div><button type="button" onClick={()=>setOpen(false)}>×</button></header>
      <div className="folder-picker-actions"><button type="button" disabled={!listing?.parent} onClick={()=>void load(listing?.parent||undefined)}>↑ Parent folder</button><button type="button" onClick={()=>void load(listing?.path||value||undefined)}>↻ Refresh</button>{listing?.roots.map(root=><button type="button" className={root.toLowerCase()===listing.path.toLowerCase()?'active':''} key={root} onClick={()=>void load(root)}>{root}</button>)}</div>
      {error?<p className="folder-picker-error">{error}</p>:<div className="folder-picker-list">{listing?.directories.map(name=><button type="button" key={name} onClick={()=>void load(`${listing.path}${listing.path.endsWith('\\')?'':'\\'}${name}`)}>▸ {name}</button>)}{listing&&!listing.directories.length&&<p>No subfolders.</p>}</div>}
      <footer><code>{listing?.path||value}</code><button type="button" className="primary-button" disabled={!listing} onClick={()=>{if(listing)onChange(listing.path);setOpen(false)}}>Use this folder</button></footer>
    </section></div>}
  </>
}

import { useEffect, useMemo, useRef, useState } from 'react'
import { Icon } from './Icon'
import { useModalA11y } from '../lib/useModalA11y'
import { rejectRelease, unrejectRelease } from '../lib/api'
import './ReleaseSearch.css'

type Release = {
  indexer_id:string; indexer_name:string; title:string; details_url:string|null; download_url:string|null;
  size_bytes:number|null; seeders:number|null; leechers:number|null; category:string|null; published:string|null;
  score:number; base_score:number; profile_score:number; match_score:number; accepted:boolean;
  rejection_id:number|null;
  reasons:string[]; rejection_reasons:string[];
  resolution:string|null; source:string|null; codec:string|null; hdr:string|null; audio:string|null; language:string|null;
}
type Response={
  status:string;results:Release[];failures:{indexer_id:string;error:string}[];searched_indexers:number;
  profile_id:number|null;profile_name:string|null;cutoff_score:number|null;
}

function size(v:number|null){if(!v)return '—';const u=['B','KB','MB','GB','TB'];let i=0,n=v;while(n>=1024&&i<u.length-1){n/=1024;i++}return `${n.toFixed(i>1?1:0)} ${u[i]}`}

export function ReleaseSearch({
  open,onClose,mediaId,title,year,tmdbId,mediaType,profileId,
  queryOverride,seasonNumber,episodeNumber,isSeasonPack=false
}:{
  open:boolean;onClose:()=>void;mediaId:number;title:string;year?:number|null;tmdbId:number;
  mediaType:'movie'|'series';profileId?:number|null;queryOverride?:string;
  seasonNumber?:number|null;episodeNumber?:number|null;isSeasonPack?:boolean
}){
  const [rows,setRows]=useState<Release[]>([])
  const [failures,setFailures]=useState<Response['failures']>([])
  const [loading,setLoading]=useState(false)
  const [error,setError]=useState('')
  const [grabbing,setGrabbing]=useState('')
  const [showRejected,setShowRejected]=useState(false)
  const [profileName,setProfileName]=useState<string|null>(null)
  const [cutoff,setCutoff]=useState<number|null>(null)
  const requestIdRef=useRef(0)
  const [reloadKey,setReloadKey]=useState(0)

  useEffect(()=>{
    if(!open)return
    const requestId=++requestIdRef.current
    const controller=new AbortController()
    setLoading(true);setError('');setRows([]);setFailures([]);setShowRejected(false)
    const q=queryOverride?.trim()||`${title}${year?` ${year}`:''}`
    const params=new URLSearchParams({query:q,media_type:mediaType,tmdb_id:String(tmdbId),media_id:String(mediaId)})
    if(profileId)params.set('profile_id',String(profileId))
    if(seasonNumber!==null&&seasonNumber!==undefined)params.set('season_number',String(seasonNumber))
    if(episodeNumber!==null&&episodeNumber!==undefined)params.set('episode_number',String(episodeNumber))
    fetch(`/api/releases/search?${params.toString()}`,{signal:controller.signal})
      .then(async r=>{if(!r.ok)throw new Error(await r.text());return r.json() as Promise<Response>})
      .then(r=>{
        if(requestIdRef.current!==requestId)return
        setRows(r.results);setFailures(r.failures);setProfileName(r.profile_name);setCutoff(r.cutoff_score)
      })
      .catch(e=>{
        if(controller.signal.aborted||requestIdRef.current!==requestId)return
        setError(e instanceof Error?e.message:String(e))
      })
      .finally(()=>{if(requestIdRef.current===requestId)setLoading(false)})
    return ()=>{controller.abort()}
  },[open,title,year,tmdbId,mediaId,mediaType,profileId,queryOverride,seasonNumber,episodeNumber,reloadKey])

  const accepted=useMemo(()=>rows.filter(x=>x.accepted),[rows])
  const rejected=useMemo(()=>rows.filter(x=>!x.accepted),[rows])
  const visible=showRejected?rows:accepted
  const best=useMemo(()=>accepted[0]?.score??null,[accepted])
  const dialogRef=useModalA11y<HTMLDivElement>(open,onClose)
  if(!open)return null

  async function grab(row:Release){
    setGrabbing(`${row.indexer_id}-${row.title}`);setError('')
    try{
      const r=await fetch('/api/releases/grab',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({
        indexer_id:row.indexer_id,indexer_name:row.indexer_name,title:row.title,
        download_url:row.download_url,details_url:row.details_url,
        category:null,
        media_type:mediaType,
        media_id:mediaId,
        profile_id:profileId??null,
        season_number:seasonNumber??null,
        episode_number:episodeNumber??null,
        is_season_pack:isSeasonPack
      })})
      if(!r.ok)throw new Error(await r.text())
      onClose()
    }catch(e){setError(e instanceof Error?e.message:String(e))}
    finally{setGrabbing('')}
  }

  // Rejects/un-rejects one result in place instead of re-running the search:
  // a manual rejection only changes this one row's accepted state.
  async function setRejection(row:Release,reject:boolean){
    const key=`${row.indexer_id}-${row.title}`
    setGrabbing(key);setError('')
    try{
      let rejectionId:number|null=null
      if(reject){
        rejectionId=(await rejectRelease({media_type:mediaType,media_id:mediaId,title:row.title,indexer_name:row.indexer_name})).id
      }else if(row.rejection_id!==null){
        await unrejectRelease(row.rejection_id)
      }
      setRows(current=>current.map(x=>x.title!==row.title?x:reject
        ?{...x,accepted:false,rejection_id:rejectionId,rejection_reasons:['Rechazado manualmente',...x.rejection_reasons]}
        // Back to "not manually rejected": the real accept/reject state needs the profile's
        // verdict again, which only a fresh search can recompute.
        :{...x,rejection_id:null,rejection_reasons:x.rejection_reasons.filter(reason=>reason!=='Rechazado manualmente')}))
      if(!reject)setReloadKey(n=>n+1)
    }catch(e){setError(e instanceof Error?e.message:String(e))}
    finally{setGrabbing('')}
  }

  return <div className="rs-backdrop" onMouseDown={onClose}>
    <div ref={dialogRef} className="rs-modal" role="dialog" aria-modal="true" aria-labelledby="release-search-title" tabIndex={-1} onMouseDown={e=>e.stopPropagation()}>
      <div className="rs-head">
        <div><p>MANUAL SEARCH</p><h2 id="release-search-title">{queryOverride||title}</h2><span>{accepted.length} accepted · {rejected.length} rejected · {failures.length} indexer errors</span>{profileName&&<em className="rs-profile">Profile: {profileName}{cutoff!==null?` · cutoff ${cutoff}`:''}</em>}</div>
        <button onClick={onClose} aria-label="Close">×</button>
      </div>
      <div className="rs-toolbar"><label><input type="checkbox" checked={showRejected} onChange={e=>setShowRejected(e.target.checked)}/> Show rejected releases</label></div>
      {error&&<div className="error-box">{error}</div>}
      {loading?<div className="rs-loading">Searching enabled indexers…</div>:
      visible.length===0?<div className="rs-loading">{rows.length?'No releases pass the selected profile.':'No releases found in the enabled indexers.'}</div>:
      <div className="rs-list">
        {visible.map((r,i)=><div className={`rs-row ${r.accepted&&r.score===best?'best':''} ${!r.accepted?'rejected':''}`} key={`${r.indexer_id}-${r.title}-${i}`}>
          <div className="rs-title"><strong>{r.title}</strong><span>{r.indexer_name}{!r.accepted?' · Rejected':''}</span>{!r.accepted&&<small>{r.rejection_reasons.join(' · ')}</small>}</div>
          <div className="rs-tags">
            {r.resolution&&<b>{r.resolution}</b>}{r.source&&<b>{r.source}</b>}{r.codec&&<b>{r.codec}</b>}{r.hdr&&<b>{r.hdr}</b>}{r.audio&&<b>{r.audio}</b>}{r.language&&<b>{r.language}</b>}
          </div>
          <div className="rs-metric"><span>Size</span><strong>{size(r.size_bytes)}</strong></div>
          <div className="rs-metric"><span>Seeds</span><strong>{r.seeders??'—'}</strong></div>
          <div className="rs-score"><span>Score</span><strong>{r.score}</strong><small>M {r.match_score} · P {r.profile_score}</small></div>
          <div className="rs-actions">
            {r.rejection_id!==null
              ?<button type="button" className="rs-reject" disabled={grabbing!==''} onClick={()=>void setRejection(r,false)} title="Allow Oberiz to pick this release again">Undo reject</button>
              :r.accepted&&<button type="button" className="rs-reject" disabled={grabbing!==''} onClick={()=>void setRejection(r,true)} title="Never pick this release again for this title">Reject</button>}
            <button className="primary-button" disabled={grabbing!==''||!r.accepted} onClick={()=>void grab(r)}>
              <Icon name="downloads" size={16}/>{grabbing===`${r.indexer_id}-${r.title}`?'Sending…':r.accepted?'Grab':'Rejected'}
            </button>
          </div>
        </div>)}
      </div>}
      {failures.length>0&&<details className="rs-errors"><summary>{failures.length} indexer errors</summary>{failures.map(x=><div key={x.indexer_id}><strong>{x.indexer_id}</strong> · {x.error}</div>)}</details>}
    </div>
  </div>
}

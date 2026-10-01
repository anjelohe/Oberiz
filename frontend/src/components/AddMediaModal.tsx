import { useEffect, useMemo, useRef, useState } from 'react'
import {
  addMovie, addSeries, getQualityProfiles, MediaSearchResult, QualityProfile,
  searchMovies, searchSeries, updateMovie, updateSeries
} from '../lib/api'
import { Icon } from './Icon'
import { useModalA11y } from '../lib/useModalA11y'

type MediaType='movie'|'series'
type Step='search'|'configure'

export function AddMediaModal({open,onClose,onAdded}:{open:boolean;onClose:()=>void;onAdded:(type:MediaType)=>void}){
  const [type,setType]=useState<MediaType>('movie')
  const [step,setStep]=useState<Step>('search')
  const [query,setQuery]=useState('')
  const [results,setResults]=useState<MediaSearchResult[]>([])
  const [selected,setSelected]=useState<MediaSearchResult|null>(null)
  const [profiles,setProfiles]=useState<QualityProfile[]>([])
  const [profileId,setProfileId]=useState<number|null>(null)
  const [monitored,setMonitored]=useState(true)
  const [monitorMode,setMonitorMode]=useState('all')
  const [loading,setLoading]=useState(false)
  const [saving,setSaving]=useState(false)
  const [error,setError]=useState('')
  // A movie and a series TMDB id are different spaces: 603 can be a real
  // movie and an unrelated real series at once. Switching `type` while a
  // search or profile load for the previous type is still in flight must
  // not let that stale response land under the new type — otherwise picking
  // a result from it carries the wrong type's tmdb_id into add().
  const requestIdRef=useRef(0)

  const enabledProfiles=useMemo(()=>profiles.filter(p=>p.enabled),[profiles])

  async function loadProfiles(nextType:MediaType,requestId:number){
    try{
      const rows=await getQualityProfiles(nextType)
      if(requestIdRef.current!==requestId)return
      setProfiles(rows)
      setProfileId(rows.find(p=>p.is_default)?.id ?? rows.find(p=>p.enabled)?.id ?? null)
    }catch(e){
      if(requestIdRef.current!==requestId)return
      setProfiles([]);setProfileId(null);setError(e instanceof Error?e.message:String(e))
    }
  }
  useEffect(()=>{if(!open)return;const requestId=++requestIdRef.current;setStep('search');setResults([]);setSelected(null);setQuery('');setMonitored(true);setMonitorMode('all');setError('');void loadProfiles(type,requestId)},[open])
  useEffect(()=>{if(!open)return;const requestId=++requestIdRef.current;setResults([]);setSelected(null);setStep('search');setError('');void loadProfiles(type,requestId)},[type])
  const dialogRef=useModalA11y<HTMLElement>(open,onClose)

  async function runSearch(){
    if(!query.trim())return
    const requestId=++requestIdRef.current
    const searchType=type
    setLoading(true);setError('')
    try{
      const rows=searchType==='movie'?await searchMovies(query.trim()):await searchSeries(query.trim())
      if(requestIdRef.current!==requestId)return
      setResults(rows)
    }catch(e){
      if(requestIdRef.current!==requestId)return
      setError(e instanceof Error?e.message:String(e))
    }finally{
      if(requestIdRef.current===requestId)setLoading(false)
    }
  }
  async function add(){if(!selected)return;setSaving(true);setError('');try{
    if(type==='movie'){const created=await addMovie(selected.tmdb_id,profileId);if(created.monitored!==monitored)await updateMovie(created.id,{monitored})}
    else{const created=await addSeries(selected.tmdb_id,profileId,monitorMode);if(created.monitored!==monitored)await updateSeries(created.id,{monitored})}
    window.dispatchEvent(new CustomEvent('oberiz-library-changed',{detail:type}));onAdded(type);onClose()
  }catch(e){setError(e instanceof Error?e.message:String(e))}finally{setSaving(false)}}
  if(!open)return null
  return <div className="add-media-overlay" onMouseDown={e=>{if(e.target===e.currentTarget)onClose()}}>
    <section ref={dialogRef} className="add-media-modal" role="dialog" aria-modal="true" aria-label="Add media" tabIndex={-1}>
      <header className="add-media-head"><div><span>LIBRARY</span><h2>Add Media</h2><p>{step==='search'?'Search TMDB for a movie or series.':'Choose how Oberiz should manage this title.'}</p></div><button className="add-media-close" onClick={onClose} aria-label="Close">×</button></header>
      <div className="add-media-type"><button className={type==='movie'?'active':''} onClick={()=>setType('movie')}><Icon name="movies" size={18}/> Movie</button><button className={type==='series'?'active':''} onClick={()=>setType('series')}><Icon name="series" size={18}/> Series</button></div>
      {step==='search'?<>
        <div className="add-media-search"><Icon name="search" size={20}/><input autoFocus value={query} onChange={e=>setQuery(e.target.value)} onKeyDown={e=>e.key==='Enter'&&void runSearch()} placeholder={`Search ${type==='movie'?'movies':'series'}...`}/><button className="primary-button" onClick={()=>void runSearch()} disabled={loading}>{loading?'Searching…':'Search'}</button></div>
        {error&&<div className="error-box">{error}</div>}
        <div className="add-media-results">{results.map(result=><button className="add-media-result" key={result.tmdb_id} onClick={()=>{setSelected(result);setStep('configure')}}>{result.poster_url?<img src={result.poster_url} alt=""/>:<div className="add-media-poster-empty"><Icon name={type==='movie'?'movies':'series'} size={27}/></div>}<div><strong>{result.title}</strong><span>{result.year??'—'} · TMDB {result.vote_average.toFixed(1)}</span><p>{result.overview||'No description available.'}</p></div><b>›</b></button>)}</div>
      </>:selected&&<>
        <div className="add-media-selected">{selected.poster_url?<img src={selected.poster_url} alt=""/>:<div className="add-media-selected-empty"><Icon name={type==='movie'?'movies':'series'} size={36}/></div>}<div><span>{type==='movie'?'MOVIE':'SERIES'}</span><h3>{selected.title}</h3><small>{selected.year??'—'} · TMDB {selected.vote_average.toFixed(1)}</small><p>{selected.overview||'No description available.'}</p></div></div>
        <div className="add-media-config"><label>Quality Profile<select value={profileId??''} onChange={e=>setProfileId(e.target.value?Number(e.target.value):null)}><option value="">No profile</option>{enabledProfiles.map(profile=><option key={profile.id} value={profile.id}>{profile.name}{profile.is_default?' · Default':''}</option>)}</select><small>The profile controls quality, language, upgrades and qBittorrent routing.</small></label>{type==='series'&&<label>Monitor Mode<select value={monitorMode} onChange={e=>setMonitorMode(e.target.value)}><option value="all">All Episodes</option><option value="future">Future Episodes</option><option value="missing">Missing Episodes</option><option value="existing">Existing Episodes</option><option value="first">First Season</option><option value="latest">Latest Season</option><option value="none">None</option></select><small>Controls which episodes automation should look for.</small></label>}<label className="add-media-monitor"><input type="checkbox" checked={monitored} onChange={e=>setMonitored(e.target.checked)}/><span><b>Monitored</b><small>{type==='movie'?'Search missing releases and allowed upgrades.':'Keep this series under automation.'}</small></span></label></div>
        {error&&<div className="error-box">{error}</div>}
        <footer className="add-media-actions"><button className="ghost-button" onClick={()=>{setStep('search');setSelected(null)}}>← Back</button><div/><button className="ghost-button" onClick={onClose}>Cancel</button><button className="primary-button" onClick={()=>void add()} disabled={saving}>{saving?'Adding…':`Add ${type==='movie'?'Movie':'Series'}`}</button></footer>
      </>}
    </section>
  </div>
}

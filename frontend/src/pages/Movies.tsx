import { t, useTranslation, tf } from '../lib/i18n'
import { useEffect, useMemo, useState } from 'react'
import { deleteMovie, getDownloadsLive, getImports, getMovies, getQualityProfiles, ImportJob, Movie, QualityProfile, TorrentLive, updateMovie } from '../lib/api'
import { Icon } from '../components/Icon'
import { ReleaseSearch } from '../components/ReleaseSearch'
import { MissingReports, MissingTarget, collectionForMovie } from '../components/MissingReports'

type ViewMode='grid'|'list'
type StatusFilter='all'|'available'|'downloading'|'missing'|'upgrade'
type MonitorFilter='all'|'monitored'|'unmonitored'

function jobFor(movie:Movie,jobs:ImportJob[]){return jobs.find(j=>j.media_type==='movie'&&j.media_id===movie.id)}
function stateFor(movie:Movie,job?:ImportJob){
  if(job?.status==='queued'||job?.status==='downloading'||job?.status==='completed')return 'downloading'
  if(movie.available)return movie.upgrade_wanted?'upgrade':'available'
  return 'missing'
}
function stateLabel(value:string){return value==='available'?t("Downloaded"):value==='downloading'?t("Downloading"):value==='upgrade'?t("Upgrade wanted"):t("Missing")}

export function Movies(){
  useTranslation()
  const [movies,setMovies]=useState<Movie[]>([])
  const [jobs,setJobs]=useState<ImportJob[]>([])
  const [torrents,setTorrents]=useState<TorrentLive[]>([])
  const [profiles,setProfiles]=useState<QualityProfile[]>([])
  const [query,setQuery]=useState('')
  const [status,setStatus]=useState<StatusFilter>('all')
  const [monitor,setMonitor]=useState<MonitorFilter>('all')
  const [quality,setQuality]=useState('all')
  const [sort,setSort]=useState("added")
  const [view,setView]=useState<ViewMode>('grid')
  const [error,setError]=useState('')
  const [downloadsError,setDownloadsError]=useState('')
  const [releaseMovie,setReleaseMovie]=useState<Movie|null>(null)
  const [missingTarget,setMissingTarget]=useState<MissingTarget|null>(null)

  async function reload(){
    const [m,j,d]=await Promise.allSettled([getMovies(),getImports(),getDownloadsLive()])
    if(m.status==='fulfilled'){setMovies(m.value);setError('')}
    else setError(m.reason instanceof Error?m.reason.message:String(m.reason))
    if(j.status==='fulfilled')setJobs(j.value)
    if(d.status==='fulfilled'){setTorrents(d.value.torrents);setDownloadsError('')}
    else setDownloadsError(d.reason instanceof Error?d.reason.message:String(d.reason))
  }
  useEffect(()=>{
    void reload()
    void getQualityProfiles('movie').then(setProfiles).catch(()=>{})
    const changed=(ev:Event)=>{if((ev as CustomEvent<string>).detail==='movie')void reload()}
    window.addEventListener('oberiz-library-changed',changed)
    // Polls the full reload, not just downloads: a job finishing import flips
    // status/available on the movie and job rows themselves, not just
    // torrent progress, and those used to only refresh on next page load.
    const timer=setInterval(()=>void reload(),5000)
    return()=>{clearInterval(timer);window.removeEventListener('oberiz-library-changed',changed)}
  },[])


  async function toggleMonitored(movie:Movie){
    setError('')
    try{await updateMovie(movie.id,!movie.monitored);await reload()}
    catch(e){setError(e instanceof Error?e.message:String(e))}
  }
  async function removeMovie(movie:Movie){
    if(!confirm(tf("Delete {0}?", movie.title)))return
    setError('')
    try{await deleteMovie(movie.id);await reload()}
    catch(e){setError(e instanceof Error?e.message:String(e))}
  }
  async function changeMovieProfile(movie:Movie,profileId:number){
    setError('')
    try{
      await updateMovie(movie.id,{quality_profile_id:profileId})
      await reload()
    }catch(e){setError(e instanceof Error?e.message:String(e))}
  }

  const visible=useMemo(()=>{
    const q=query.trim().toLowerCase()
    let list=movies.filter(m=>{
      const state=stateFor(m,jobFor(m,jobs))
      const qualityOk=quality==='all'||String(m.quality_profile_id??'')===quality
      const monitorOk=monitor==='all'||(monitor==='monitored'?m.monitored:!m.monitored)
      return (!q||`${m.title} ${m.original_title??''} ${m.year??''} ${m.quality_profile_name??''}`.toLowerCase().includes(q))
        && (status==='all'||state===status) && monitorOk && qualityOk
    })
    return [...list].sort((a,b)=>sort==='title'?a.title.localeCompare(b.title):sort==='year'?(b.year??0)-(a.year??0):b.id-a.id)
  },[movies,jobs,query,status,monitor,quality,sort])

  const monitored=movies.filter(m=>m.monitored).length
  const downloaded=movies.filter(m=>m.available).length
  const wanted=movies.filter(m=>m.monitored&&!m.available).length
  const upgrades=movies.filter(m=>m.upgrade_wanted).length
  const recentlyAdded=[...movies].sort((a,b)=>b.id-a.id).slice(0,5)
  const torrentFor=(job?:ImportJob)=>job?.qb_hash?torrents.find(t=>t.hash===job.qb_hash):undefined
  const fmtSpeed=(value:number)=>{
    if(!value)return '0 B/s'
    const units=['B/s','KB/s','MB/s','GB/s'];let n=value,i=0
    while(n>=1024&&i<units.length-1){n/=1024;i++}
    return `${n.toFixed(i<2?0:1)} ${units[i]}`
  }

  return <div className="lib-page movie-lib-page">
    {missingTarget&&<MissingReports key={`${missingTarget.scope}-${missingTarget.tmdb_id??''}`} target={missingTarget} onClose={()=>setMissingTarget(null)}/>}
    <section className="lib-hero">
      <div className="lib-title"><div className="lib-title-icon"><Icon name="movies" size={31}/></div><div><h1>{t("Movies Library")}</h1><p>{t("Manage your movie collection, downloads and monitoring")}</p></div></div>
    </section>

    {error&&<div className="error-box">{error}</div>}
    {downloadsError&&<div className="error-box">{t("qBittorrent unavailable:")} {downloadsError}{t(". Library metadata stays usable; download progress is paused.")}</div>}

    <section className="lib-stats">
      <div className="lib-stat"><span className="lib-stat-icon cyan"><Icon name="movies"/></span><div><small>{t("Total Movies")}</small><strong>{movies.length}</strong><em>{t("Library entries")}</em></div></div>
      <div className="lib-stat"><span className="lib-stat-icon green">◉</span><div><small>{t("Monitored Movies")}</small><strong>{monitored}</strong><em>{movies.length?Math.round(monitored/movies.length*100):0}{t("% of library")}</em></div></div>
      <div className="lib-stat"><span className="lib-stat-icon blue"><Icon name="downloads"/></span><div><small>{t("Downloaded")}</small><strong>{downloaded}</strong><em>{movies.length?Math.round(downloaded/movies.length*100):0}{t("% of library")}</em></div></div>
      <div className="lib-stat"><span className="lib-stat-icon yellow">⌑</span><div><small>{t("Wanted / Upgrades")}</small><strong>{wanted} / {upgrades}</strong><em>{t("Missing · quality upgrades")}</em></div></div>
    </section>


    <section className="lib-toolbar">
      <label className="lib-search"><Icon name="search" size={17}/><input value={query} onChange={e=>setQuery(e.target.value)} placeholder={t("Search movies...")}/></label>
      <select value={status} onChange={e=>setStatus(e.target.value as StatusFilter)}><option value="all">{t("All Availability")}</option><option value="available">{t("Downloaded")}</option><option value="downloading">{t("Downloading")}</option><option value="missing">{t("Missing")}</option><option value="upgrade">{t("Upgrade wanted")}</option></select>
      <select value={monitor} onChange={e=>setMonitor(e.target.value as MonitorFilter)}><option value="all">{t("All Monitoring")}</option><option value="monitored">{t("Monitored")}</option><option value="unmonitored">{t("Not Monitored")}</option></select>
      <select value={quality} onChange={e=>setQuality(e.target.value)}><option value="all">{t("All Quality")}</option>{profiles.map(p=><option key={p.id} value={p.id}>{p.name}</option>)}</select>
      <select value={sort} onChange={e=>setSort(e.target.value)}><option value="added">{t("Sort by: Date Added")}</option><option value="title">{t("Sort by: Title")}</option><option value="year">{t("Sort by: Year")}</option></select>
      <div className="lib-view"><button className={view==='grid'?'active':''} onClick={()=>setView('grid')}>▦</button><button className={view==='list'?'active':''} onClick={()=>setView('list')}>☷</button></div>
      <button type="button" className="library-report-action" onClick={()=>setMissingTarget({scope:'collections'})}><span className="library-report-icon"><Icon name="imports" size={17}/></span><span>{t('Incomplete collections')}</span><span className="library-report-arrow"><Icon name="chevronDown" size={13}/></span></button>
    </section>

    <div className="movie-lib-layout">
      <div className={`poster-library-grid ${view}`}>
        {visible.map(movie=>{
          const job=jobFor(movie,jobs), state=stateFor(movie,job)
          const torrent=torrentFor(job)
          const liveProgress=torrent?Math.max(0,Math.min(100,torrent.progress*100)):movie.available?100:0
          return <article className="poster-library-card" key={movie.id}>
            <div className="poster-library-art">
              {movie.poster_path?<img src={movie.poster_path} alt={movie.title}/>:<div className="poster-library-placeholder"><Icon name="movies" size={36}/></div>}
              <span className={`poster-state ${state}`}>{state==='available'?'✓ ':state==='downloading'?'↓ ':state==='upgrade'?'↑ ':'! '}{stateLabel(state)}</span>
              <span className={`poster-monitor-state ${movie.monitored?'on':'off'}`}>{movie.monitored?t("● Monitored"):t("○ Not monitored")}</span>
              <button className="poster-kebab" onClick={()=>setReleaseMovie(movie)}>⋮</button>
            </div>
            <div className="poster-library-copy">
              <h3>{movie.title}</h3><span className="poster-year">{movie.year??'—'}</span>
              <div className="poster-chips">
                <select
                  className="poster-profile-select"
                  value={movie.quality_profile_id??''}
                  title={t("Quality Profile")}
                  onChange={e=>e.target.value&&void changeMovieProfile(movie,Number(e.target.value))}
                >
                  {!movie.quality_profile_id&&<option value="">{t("Default profile")}</option>}
                  {profiles.filter(p=>p.enabled||p.id===movie.quality_profile_id).map(p=><option key={p.id} value={p.id}>{p.name}</option>)}
                </select>
                {movie.current_resolution&&<b>{movie.current_resolution}</b>}
                {job?.import_method&&<b>{job.import_method}</b>}
                {torrent&&state==='downloading'&&<b>{liveProgress.toFixed(1)}%</b>}
              </div>
              {torrent&&state==='downloading'&&<><div className="media-live-progress"><i style={{width:`${liveProgress}%`}}/></div><div className="media-live-meta"><span>{fmtSpeed(torrent.dlspeed)}</span><span>{torrent.num_seeds} {t("seeds")}</span></div></>}
              <div className="poster-actions">
                <button title={t('Check collection')} aria-label={t('Check collection')} onClick={()=>void collectionForMovie(movie.tmdb_id).then(setMissingTarget).catch(e=>setError(e.message))}><Icon name="imports" size={16}/></button>
                <button title={t("Search releases")} onClick={()=>setReleaseMovie(movie)}>▶</button>
                <button title={movie.monitored?t("Unmonitor"):t("Monitor")} onClick={()=>void toggleMonitored(movie)}>{movie.monitored?'♡':'○'}</button>
                <button title={t("Details/profile")} onClick={()=>setReleaseMovie(movie)}>ⓘ</button>
                <button title={t("Delete")} onClick={()=>void removeMovie(movie)}>⌫</button>
              </div>
            </div>
          </article>
        })}
        {!visible.length&&<div className="empty-state card">{t("No movies match the current filters.")}</div>}
      </div>

      <aside className="lib-side">
        <section className="lib-side-card">
          <header><strong>{t("Library Overview")}</strong></header>
          <div className="overview-line"><i className="green"/><span>{t("Monitored")}</span><b>{monitored}</b><em>{movies.length?Math.round(monitored/movies.length*100):0}%</em></div>
          <div className="overview-line"><i className="cyan"/><span>{t("Downloaded")}</span><b>{downloaded}</b><em>{movies.length?Math.round(downloaded/movies.length*100):0}%</em></div>
          <div className="overview-line"><i className="yellow"/><span>{t("Wanted")}</span><b>{wanted}</b><em>{movies.length?Math.round(wanted/movies.length*100):0}%</em></div>
          <div className="overview-line"><i className="yellow"/><span>{t("Upgrades wanted")}</span><b>{upgrades}</b><em>{movies.length?Math.round(upgrades/movies.length*100):0}%</em></div>
        </section>
        <section className="lib-side-card">
          <header><strong>{t("Recently Added")}</strong></header>
          <div className="recent-poster-row">{recentlyAdded.map(m=><button key={m.id} onClick={()=>setReleaseMovie(m)}>{m.poster_path?<img src={m.poster_path} alt=""/>:<span/>}<small>{m.title}</small></button>)}</div>
        </section>
        <section className="lib-side-card">
          <header><strong>{t("Profiles")}</strong></header>
          {profiles.slice(0,6).map(p=><div className="profile-line" key={p.id}><span>{p.name}</span><b>{movies.filter(m=>m.quality_profile_id===p.id).length}</b></div>)}
        </section>
      </aside>
    </div>

    <ReleaseSearch open={!!releaseMovie} onClose={()=>setReleaseMovie(null)} mediaId={releaseMovie?.id??0} title={releaseMovie?.title??''} year={releaseMovie?.year} tmdbId={releaseMovie?.tmdb_id??0} mediaType="movie" profileId={releaseMovie?.quality_profile_id??null}/>
  </div>
}

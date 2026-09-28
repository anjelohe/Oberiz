import { useEffect, useMemo, useState } from 'react'
import { deleteMovie, getDownloadsLive, getImports, getMovies, getQualityProfiles, ImportJob, Movie, QualityProfile, TorrentLive, updateMovie } from '../lib/api'
import { Icon } from '../components/Icon'
import { ReleaseSearch } from '../components/ReleaseSearch'

type ViewMode='grid'|'list'
type StatusFilter='all'|'available'|'downloading'|'missing'|'upgrade'
type MonitorFilter='all'|'monitored'|'unmonitored'

function jobFor(movie:Movie,jobs:ImportJob[]){return jobs.find(j=>j.media_type==='movie'&&j.media_id===movie.id)}
function stateFor(movie:Movie,job?:ImportJob){
  if(job?.status==='queued'||job?.status==='downloading'||job?.status==='completed')return 'downloading'
  if(movie.available)return movie.upgrade_wanted?'upgrade':'available'
  return 'missing'
}
function stateLabel(value:string){return value==='available'?'Downloaded':value==='downloading'?'Downloading':value==='upgrade'?'Upgrade wanted':'Missing'}

export function Movies(){
  const [movies,setMovies]=useState<Movie[]>([])
  const [jobs,setJobs]=useState<ImportJob[]>([])
  const [torrents,setTorrents]=useState<TorrentLive[]>([])
  const [profiles,setProfiles]=useState<QualityProfile[]>([])
  const [query,setQuery]=useState('')
  const [status,setStatus]=useState<StatusFilter>('all')
  const [monitor,setMonitor]=useState<MonitorFilter>('all')
  const [quality,setQuality]=useState('all')
  const [sort,setSort]=useState('added')
  const [view,setView]=useState<ViewMode>('grid')
  const [error,setError]=useState('')
  const [releaseMovie,setReleaseMovie]=useState<Movie|null>(null)

  async function reload(){
    const [m,j,d]=await Promise.all([getMovies(),getImports(),getDownloadsLive()])
    setMovies(m);setJobs(j);setTorrents(d.torrents)
  }
  useEffect(()=>{
    void reload().catch(e=>setError(String(e)))
    void getQualityProfiles('movie').then(setProfiles).catch(()=>{})
    const changed=(ev:Event)=>{if((ev as CustomEvent<string>).detail==='movie')void reload()}
    window.addEventListener('oberiz-library-changed',changed)
    const timer=setInterval(()=>void getDownloadsLive().then(d=>setTorrents(d.torrents)).catch(()=>{}),5000)
    return()=>{clearInterval(timer);window.removeEventListener('oberiz-library-changed',changed)}
  },[])


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
    <section className="lib-hero">
      <div className="lib-title"><div className="lib-title-icon"><Icon name="movies" size={31}/></div><div><h1>Movies Library</h1><p>Manage your movie collection, downloads and monitoring</p></div></div>
    </section>

    <section className="lib-stats">
      <div className="lib-stat"><span className="lib-stat-icon cyan"><Icon name="movies"/></span><div><small>Total Movies</small><strong>{movies.length}</strong><em>Library entries</em></div></div>
      <div className="lib-stat"><span className="lib-stat-icon green">◉</span><div><small>Monitored Movies</small><strong>{monitored}</strong><em>{movies.length?Math.round(monitored/movies.length*100):0}% of library</em></div></div>
      <div className="lib-stat"><span className="lib-stat-icon blue"><Icon name="downloads"/></span><div><small>Downloaded</small><strong>{downloaded}</strong><em>{movies.length?Math.round(downloaded/movies.length*100):0}% of library</em></div></div>
      <div className="lib-stat"><span className="lib-stat-icon yellow">⌑</span><div><small>Wanted / Upgrades</small><strong>{wanted} / {upgrades}</strong><em>Missing · quality upgrades</em></div></div>
    </section>


    <section className="lib-toolbar">
      <label className="lib-search"><Icon name="search" size={17}/><input value={query} onChange={e=>setQuery(e.target.value)} placeholder="Search movies..."/></label>
      <select value={status} onChange={e=>setStatus(e.target.value as StatusFilter)}><option value="all">All Availability</option><option value="available">Downloaded</option><option value="downloading">Downloading</option><option value="missing">Missing</option><option value="upgrade">Upgrade wanted</option></select>
      <select value={monitor} onChange={e=>setMonitor(e.target.value as MonitorFilter)}><option value="all">All Monitoring</option><option value="monitored">Monitored</option><option value="unmonitored">Not Monitored</option></select>
      <select value={quality} onChange={e=>setQuality(e.target.value)}><option value="all">All Quality</option>{profiles.map(p=><option key={p.id} value={p.id}>{p.name}</option>)}</select>
      <select value={sort} onChange={e=>setSort(e.target.value)}><option value="added">Sort by: Date Added</option><option value="title">Sort by: Title</option><option value="year">Sort by: Year</option></select>
      <div className="lib-view"><button className={view==='grid'?'active':''} onClick={()=>setView('grid')}>▦</button><button className={view==='list'?'active':''} onClick={()=>setView('list')}>☷</button></div>
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
              <span className={`poster-monitor-state ${movie.monitored?'on':'off'}`}>{movie.monitored?'● Monitored':'○ Not monitored'}</span>
              <button className="poster-kebab" onClick={()=>setReleaseMovie(movie)}>⋮</button>
            </div>
            <div className="poster-library-copy">
              <h3>{movie.title}</h3><span className="poster-year">{movie.year??'—'}</span>
              <div className="poster-chips">
                <select
                  className="poster-profile-select"
                  value={movie.quality_profile_id??''}
                  title="Quality Profile"
                  onChange={e=>e.target.value&&void changeMovieProfile(movie,Number(e.target.value))}
                >
                  {!movie.quality_profile_id&&<option value="">Default profile</option>}
                  {profiles.filter(p=>p.enabled||p.id===movie.quality_profile_id).map(p=><option key={p.id} value={p.id}>{p.name}</option>)}
                </select>
                {movie.current_resolution&&<b>{movie.current_resolution}</b>}
                {job?.import_method&&<b>{job.import_method}</b>}
                {torrent&&state==='downloading'&&<b>{liveProgress.toFixed(1)}%</b>}
              </div>
              {torrent&&state==='downloading'&&<><div className="media-live-progress"><i style={{width:`${liveProgress}%`}}/></div><div className="media-live-meta"><span>{fmtSpeed(torrent.dlspeed)}</span><span>{torrent.num_seeds} seeds</span></div></>}
              <div className="poster-actions">
                <button title="Search releases" onClick={()=>setReleaseMovie(movie)}>▶</button>
                <button title={movie.monitored?'Unmonitor':'Monitor'} onClick={async()=>{await updateMovie(movie.id,!movie.monitored);await reload()}}>{movie.monitored?'♡':'○'}</button>
                <button title="Details/profile" onClick={()=>setReleaseMovie(movie)}>ⓘ</button>
                <button title="Delete" onClick={async()=>{if(confirm(`Delete ${movie.title}?`)){await deleteMovie(movie.id);await reload()}}}>⌫</button>
              </div>
            </div>
          </article>
        })}
        {!visible.length&&<div className="empty-state card">No movies match the current filters.</div>}
      </div>

      <aside className="lib-side">
        <section className="lib-side-card">
          <header><strong>Library Overview</strong></header>
          <div className="overview-line"><i className="green"/><span>Monitored</span><b>{monitored}</b><em>{movies.length?Math.round(monitored/movies.length*100):0}%</em></div>
          <div className="overview-line"><i className="cyan"/><span>Downloaded</span><b>{downloaded}</b><em>{movies.length?Math.round(downloaded/movies.length*100):0}%</em></div>
          <div className="overview-line"><i className="yellow"/><span>Wanted</span><b>{wanted}</b><em>{movies.length?Math.round(wanted/movies.length*100):0}%</em></div>
          <div className="overview-line"><i className="yellow"/><span>Upgrades wanted</span><b>{upgrades}</b><em>{movies.length?Math.round(upgrades/movies.length*100):0}%</em></div>
        </section>
        <section className="lib-side-card">
          <header><strong>Recently Added</strong></header>
          <div className="recent-poster-row">{recentlyAdded.map(m=><button key={m.id} onClick={()=>setReleaseMovie(m)}>{m.poster_path?<img src={m.poster_path} alt=""/>:<span/>}<small>{m.title}</small></button>)}</div>
        </section>
        <section className="lib-side-card">
          <header><strong>Profiles</strong></header>
          {profiles.slice(0,6).map(p=><div className="profile-line" key={p.id}><span>{p.name}</span><b>{movies.filter(m=>m.quality_profile_id===p.id).length}</b></div>)}
        </section>
      </aside>
    </div>

    <ReleaseSearch open={!!releaseMovie} onClose={()=>setReleaseMovie(null)} mediaId={releaseMovie?.id??0} title={releaseMovie?.title??''} year={releaseMovie?.year} tmdbId={releaseMovie?.tmdb_id??0} mediaType="movie" profileId={releaseMovie?.quality_profile_id??null}/>
  </div>
}

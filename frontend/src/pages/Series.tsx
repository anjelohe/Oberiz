import { useEffect, useMemo, useState } from 'react'
import {
  deleteSeries, getDownloadsLive, getImports, getQualityProfiles, getSeries,
  getSeriesDetail, ImportJob, QualityProfile, refreshSeries,
  Series as SeriesType, SeriesDetailResponse, SeriesEpisode, SeasonWithEpisodes, TorrentLive,
  updateSeries, updateSeriesEpisode, updateSeriesSeason
} from '../lib/api'
import { Icon } from '../components/Icon'
import { ReleaseSearch } from '../components/ReleaseSearch'

type StatusFilter='all'|'monitored'|'unmonitored'|'available'|'downloading'
type SearchTarget={
  mediaId:number
  tmdbId:number
  title:string
  query:string
  profileId:number|null
  seasonNumber:number|null
  episodeNumber:number|null
  isSeasonPack:boolean
}|null

function jobFor(item:SeriesType,jobs:ImportJob[]){
  return jobs.find(j=>j.media_type==='series'&&j.media_id===item.id)
}
function stateFor(item:SeriesType,job?:ImportJob){
  if(job?.status==='downloading'||job?.status==='completed')return 'downloading'
  if(job?.status==='seeding'||job?.status==='cleaned')return 'available'
  return item.monitored?'monitored':'unmonitored'
}
function monitorLabel(mode:string){
  return ({
    all:'All episodes',future:'Future episodes',missing:'Missing episodes',
    existing:'Existing episodes',first:'First season',latest:'Latest season',none:'None'
  } as Record<string,string>)[mode]||mode
}
function episodeCode(ep:SeriesEpisode){return `S${String(ep.season_number).padStart(2,'0')}E${String(ep.episode_number).padStart(2,'0')}`}
function code(season:number|null,episode:number|null){return season!=null&&episode!=null?`S${String(season).padStart(2,'0')}E${String(episode).padStart(2,'0')}`:''}
function shortDate(value:string|null){if(!value)return '';const d=new Date(`${value}T00:00:00`);return Number.isNaN(d.getTime())?value:d.toLocaleDateString(undefined,{month:'short',day:'numeric'})}

export function Series(){
  const [items,setItems]=useState<SeriesType[]>([])
  const [jobs,setJobs]=useState<ImportJob[]>([])
  const [torrents,setTorrents]=useState<TorrentLive[]>([])
  const [profiles,setProfiles]=useState<QualityProfile[]>([])
  const [query,setQuery]=useState('')
  const [status,setStatus]=useState<StatusFilter>('all')
  const [sort,setSort]=useState('added')
  const [error,setError]=useState('')
  const [expanded,setExpanded]=useState<number|null>(null)
  const [details,setDetails]=useState<Record<number,SeriesDetailResponse>>({})
  const [expandedSeasons,setExpandedSeasons]=useState<Record<string,boolean>>({})
  const [searchTarget,setSearchTarget]=useState<SearchTarget>(null)
  const [busy,setBusy]=useState('')

  async function reload(){
    const [s,j,d]=await Promise.all([getSeries(),getImports(),getDownloadsLive()])
    setItems(s);setJobs(j);setTorrents(d.torrents)
  }
  async function loadDetail(id:number){
    const detail=await getSeriesDetail(id)
    setDetails(current=>({...current,[id]:detail}))
    return detail
  }

  useEffect(()=>{
    void reload().catch(e=>setError(String(e)))
    void getQualityProfiles('series').then(setProfiles).catch(()=>{})
    const changed=(ev:Event)=>{if((ev as CustomEvent<string>).detail==='series')void reload()}
    window.addEventListener('oberiz-library-changed',changed)
    const timer=setInterval(()=>void getDownloadsLive().then(d=>setTorrents(d.torrents)).catch(()=>{}),5000)
    return()=>{clearInterval(timer);window.removeEventListener('oberiz-library-changed',changed)}
  },[])


  const visible=useMemo(()=>{
    const q=query.trim().toLowerCase()
    let list=items.filter(item=>{
      const state=stateFor(item,jobFor(item,jobs))
      return (!q||`${item.name} ${item.original_name??''} ${item.year??''} ${item.quality_profile_name??''}`.toLowerCase().includes(q))
        && (status==='all'||state===status)
    })
    return [...list].sort((a,b)=>sort==='title'?a.name.localeCompare(b.name):sort==='year'?(b.year??0)-(a.year??0):b.id-a.id)
  },[items,jobs,query,status,sort])

  const monitored=items.filter(x=>x.monitored).length
  const monitoredEpisodes=items.reduce((sum,x)=>sum+(x.monitored_episode_count||0),0)
  const upcomingEpisodes=items.reduce((sum,x)=>sum+(x.future_episode_count||0),0)
  const available=items.filter(x=>x.episode_count>0&&x.available_episode_count>=x.episode_count).length
  const downloading=items.filter(x=>stateFor(x,jobFor(x,jobs))==='downloading').length
  const recent=[...items].sort((a,b)=>b.id-a.id).slice(0,5)

  const torrentFor=(job?:ImportJob)=>job?.qb_hash?torrents.find(t=>t.hash===job.qb_hash):undefined
  const fmtSpeed=(value:number)=>{
    if(!value)return '0 B/s'
    const units=['B/s','KB/s','MB/s','GB/s'];let n=value,i=0
    while(n>=1024&&i<units.length-1){n/=1024;i++}
    return `${n.toFixed(i<2?0:1)} ${units[i]}`
  }
  const fmtEta=(seconds:number)=>{
    if(!Number.isFinite(seconds)||seconds<=0||seconds>=8640000)return '∞'
    const h=Math.floor(seconds/3600),m=Math.floor((seconds%3600)/60)
    return h?`${h}h ${m}m`:`${Math.max(1,m)}m`
  }

  async function toggleExpanded(item:SeriesType){
    if(expanded===item.id){setExpanded(null);return}
    setExpanded(item.id)
    if(!details[item.id]){
      try{await loadDetail(item.id)}catch(e){setError(e instanceof Error?e.message:String(e))}
    }
  }

  async function refreshMetadata(item:SeriesType){
    setBusy(`refresh-${item.id}`)
    try{
      const detail=await refreshSeries(item.id)
      setDetails(current=>({...current,[item.id]:detail}))
      await reload()
    }catch(e){setError(e instanceof Error?e.message:String(e))}
    finally{setBusy('')}
  }

  async function changeMonitorMode(item:SeriesType,mode:string){
    setBusy(`mode-${item.id}`)
    try{
      await updateSeries(item.id,{monitor_mode:mode})
      await Promise.all([reload(),loadDetail(item.id)])
    }catch(e){setError(e instanceof Error?e.message:String(e))}
    finally{setBusy('')}
  }

  async function changeSeriesProfile(item:SeriesType,profileId:number){
    try{
      await updateSeries(item.id,{quality_profile_id:profileId})
      await Promise.all([reload(),loadDetail(item.id)])
    }catch(e){setError(e instanceof Error?e.message:String(e))}
  }

  async function changeSeason(item:SeriesType,season:SeasonWithEpisodes,payload:{monitored?:boolean;quality_profile_id?:number|null;clear_quality_profile?:boolean}){
    setBusy(`season-${item.id}-${season.season_number}`)
    try{
      await updateSeriesSeason(item.id,season.season_number,payload)
      await loadDetail(item.id)
    }catch(e){setError(e instanceof Error?e.message:String(e))}
    finally{setBusy('')}
  }

  async function changeEpisode(item:SeriesType,episode:SeriesEpisode,payload:{monitored?:boolean;quality_profile_id?:number|null;clear_quality_profile?:boolean}){
    setBusy(`episode-${episode.id}`)
    try{
      await updateSeriesEpisode(item.id,episode.id,payload)
      await loadDetail(item.id)
    }catch(e){setError(e instanceof Error?e.message:String(e))}
    finally{setBusy('')}
  }

  return <div className="lib-page series-lib-page">
    <section className="lib-hero">
      <div className="lib-title"><div className="lib-title-icon"><Icon name="series" size={31}/></div><div><h1>Series Library</h1><p>Track seasons and episodes with inherited profiles and monitoring rules</p></div></div>
    </section>

    <section className="lib-stats">
      <div className="lib-stat"><span className="lib-stat-icon cyan"><Icon name="series"/></span><div><small>Total Series</small><strong>{items.length}</strong><em>{monitored} monitored</em></div></div>
      <div className="lib-stat"><span className="lib-stat-icon green">◉</span><div><small>Episodes Monitored</small><strong>{monitoredEpisodes}</strong><em>Automation scope</em></div></div>
      <div className="lib-stat"><span className="lib-stat-icon blue">▣</span><div><small>Upcoming Episodes</small><strong>{upcomingEpisodes}</strong><em>Future monitored episodes</em></div></div>
      <div className="lib-stat"><span className="lib-stat-icon yellow"><Icon name="downloads"/></span><div><small>Active Downloads</small><strong>{downloading}</strong><em>Current jobs</em></div></div>
    </section>


    {error&&<div className="error-box">{error}</div>}

    <section className="series-filterbar">
      <div className="series-filterchips">
        <button className={status==='all'?'active':''} onClick={()=>setStatus('all')}>All Series</button>
        <button className={status==='monitored'?'active':''} onClick={()=>setStatus('monitored')}>Monitored</button>
        <button className={status==='available'?'active':''} onClick={()=>setStatus('available')}>Completed</button>
        <button className={status==='downloading'?'active':''} onClick={()=>setStatus('downloading')}>Downloading</button>
        <button className={status==='unmonitored'?'active':''} onClick={()=>setStatus('unmonitored')}>On Hold</button>
      </div>
      <select value={sort} onChange={e=>setSort(e.target.value)}><option value="added">Sort by: Recently Added</option><option value="title">Sort by: Title</option><option value="year">Sort by: Year</option></select>
      <label className="lib-search"><Icon name="search" size={17}/><input value={query} onChange={e=>setQuery(e.target.value)} placeholder="Search series..."/></label>
    </section>

    <div className="series-layout">
      <div className="series-list">
        {visible.map(item=>{
          const job=jobFor(item,jobs), state=stateFor(item,job)
          const torrent=torrentFor(job)
          const liveProgress=torrent?Math.max(0,Math.min(100,torrent.progress*100)):state==='available'?100:0
          const statusText=state==='available'?'Up to date':state==='downloading'?'Downloading':item.monitored?'Monitored':'On hold'
          const detail=details[item.id]
          const seasonRows=detail?.seasons??[]
          const regularSeasons=seasonRows.filter(s=>s.season_number>0)
          const totalEpisodes=detail?regularSeasons.reduce((a,s)=>a+s.episode_count,0):item.episode_count
          const availableEpisodes=detail?regularSeasons.reduce((a,s)=>a+s.available_episodes,0):item.available_episode_count
          const seasonCount=detail?regularSeasons.length:item.season_count
          const libraryPct=totalEpisodes?Math.round(availableEpisodes/totalEpisodes*100):0
          const missing=item.missing_episode_count||0
          const future=item.future_episode_count||0
          const missingCode=code(item.next_missing_season,item.next_missing_episode)
          const upcomingCode=code(item.next_upcoming_season,item.next_upcoming_episode)

          return <div className="series-entry" key={item.id}>
            <article className={`series-row-card ${expanded===item.id?'expanded':''}`}>
              <button className="series-row-poster" onClick={()=>void toggleExpanded(item)}>{item.poster_path?<img src={item.poster_path} alt={item.name}/>:<div><Icon name="series" size={32}/></div>}</button>
              <div className="series-row-main">
                <div className="series-row-title"><button className="series-title-button" onClick={()=>void toggleExpanded(item)}><h3>{item.name}</h3></button><span className={`series-chip ${state}`}>● {statusText}</span></div>
                <div className="series-row-meta"><span>{item.year??'—'}</span><i/><span>{item.quality_profile_name||'Default profile'}</span><i/><span>{seasonCount} season{seasonCount===1?'':'s'} · {totalEpisodes} episodes</span><i/><span>{monitorLabel(item.monitor_mode)}</span></div>
                <p>{item.overview||'No overview available.'}</p>
              </div>
              <div className="series-row-state series-summary-state">
                <div className="series-summary-head"><strong>{seasonCount?`${seasonCount} season${seasonCount===1?'':'s'}`:'Metadata'}</strong><span>{availableEpisodes} / {totalEpisodes} episodes</span></div>
                <div className="series-progress"><i style={{width:`${state==='downloading'?liveProgress:libraryPct}%`}}/></div>
                <div className={`series-state-box ${state}`}>
                  <span>{state==='downloading'?'↓':missing>0?'!':future>0?'▣':'✓'}</span>
                  <div>
                    <b>{state==='downloading'?`Downloading ${liveProgress.toFixed(1)}%`:missing>0?`${missing} missing episode${missing===1?'':'s'}`:future>0?'Next episode':'Up to date'}</b>
                    <small>{torrent&&state==='downloading'?`${fmtSpeed(torrent.dlspeed)} · ETA ${fmtEta(torrent.eta)} · ${torrent.num_seeds} seeds`:missing>0?`${missingCode}${item.next_missing_name?` · ${item.next_missing_name}`:''}`:future>0?`${upcomingCode}${item.next_upcoming_name?` · ${item.next_upcoming_name}`:''}${item.next_upcoming_air_date?` · ${shortDate(item.next_upcoming_air_date)}`:''}`:totalEpisodes?`${availableEpisodes}/${totalEpisodes} available`:'Expand to load episode metadata'}</small>
                  </div>
                </div>
              </div>
              <div className="series-row-actions">
                <button title="Open seasons" aria-label="Open seasons" onClick={()=>void toggleExpanded(item)}><Icon name={expanded===item.id?'chevronUp':'chevronDown'} size={18}/></button>
                <button title="Series release search" aria-label="Series release search" onClick={()=>setSearchTarget({mediaId:item.id,tmdbId:item.tmdb_id,title:item.name,query:`${item.name}${item.year?` ${item.year}`:''}`,profileId:item.quality_profile_id??null,seasonNumber:null,episodeNumber:null,isSeasonPack:false})}><Icon name="play" size={16}/></button>
                <button title="More" aria-label="More series options" onClick={()=>void toggleExpanded(item)}><Icon name="more" size={18}/></button>
              </div>
            </article>

            {expanded===item.id&&<section className="series-detail-panel">
              {!detail?<div className="series-detail-loading">Loading seasons…</div>:<>
                <div className="series-detail-toolbar">
                  <label>Monitor
                    <select value={item.monitor_mode} disabled={busy===`mode-${item.id}`} onChange={e=>void changeMonitorMode(item,e.target.value)}>
                      <option value="all">All Episodes</option>
                      <option value="future">Future Episodes</option>
                      <option value="missing">Missing Episodes</option>
                      <option value="existing">Existing Episodes</option>
                      <option value="first">First Season</option>
                      <option value="latest">Latest Season</option>
                      <option value="none">None</option>
                    </select>
                  </label>
                  <label>Default Series Profile
                    <select value={item.quality_profile_id??''} onChange={e=>e.target.value&&void changeSeriesProfile(item,Number(e.target.value))}>
                      <option value="">No profile</option>{profiles.map(p=><option key={p.id} value={p.id}>{p.name}</option>)}
                    </select>
                  </label>
                  <button className="ghost-button" disabled={busy===`refresh-${item.id}`} onClick={()=>void refreshMetadata(item)}>{busy===`refresh-${item.id}`?'Refreshing…':'Refresh TMDB'}</button>
                  <button className="danger-button" onClick={async()=>{if(confirm(`Delete ${item.name}?`)){await deleteSeries(item.id);setExpanded(null);await reload()}}}>Delete Series</button>
                </div>

                <div className="season-list">
                  {detail.seasons.map(season=>{
                    const seasonKey=`${item.id}-${season.season_number}`
                    const open=expandedSeasons[seasonKey]??false
                    const pct=season.episode_count?Math.round(season.available_episodes/season.episode_count*100):0
                    return <article className={`season-card ${season.monitored?'monitored':''}`} key={season.id}>
                      <div className="season-summary">
                        <button className="season-expand" onClick={()=>setExpandedSeasons(v=>({...v,[seasonKey]:!open}))}>{open?'⌄':'›'}</button>
                        <div className="season-poster">{season.poster_path?<img src={season.poster_path} alt=""/>:<span>{season.season_number===0?'SP':`S${season.season_number}`}</span>}</div>
                        <div className="season-copy">
                          <div><h4>{season.season_number===0?'Specials':season.name||`Season ${season.season_number}`}</h4><span>{season.air_date||'No air date'}</span></div>
                          <p>{season.overview||`${season.episode_count} episodes`}</p>
                          <div className="season-progress"><i style={{width:`${pct}%`}}/></div>
                          <small>{season.available_episodes} / {season.episode_count} available · {season.monitored_episodes} monitored</small>
                        </div>
                        <label className="season-monitor"><input type="checkbox" checked={season.monitored} onChange={e=>void changeSeason(item,season,{monitored:e.target.checked})}/> Monitor</label>
                        <label className="season-profile">Profile
                          <select value={season.quality_profile_id??''} onChange={e=>{
                            if(e.target.value)void changeSeason(item,season,{quality_profile_id:Number(e.target.value)})
                            else void changeSeason(item,season,{clear_quality_profile:true})
                          }}>
                            <option value="">Inherit: {season.effective_quality_profile_name||'None'}</option>
                            {profiles.map(p=><option key={p.id} value={p.id}>{p.name}</option>)}
                          </select>
                        </label>
                        <button className="ghost-button" disabled={season.season_number===0} onClick={()=>setSearchTarget({
                          mediaId:item.id,tmdbId:item.tmdb_id,
                          title:`${item.name} Season ${season.season_number}`,
                          query:`${item.name} S${String(season.season_number).padStart(2,'0')}`,
                          profileId:season.effective_quality_profile_id,
                          seasonNumber:season.season_number,episodeNumber:null,isSeasonPack:true
                        })}>Search Pack</button>
                      </div>

                      {open&&<div className="episode-table">
                        <div className="episode-head"><span>Episode</span><span>Air Date</span><span>Status</span><span>Profile</span><span>Monitor</span><span>Action</span></div>
                        {season.episodes.map(ep=><div className="episode-row" key={ep.id}>
                          <div className="episode-name">{ep.still_path?<img src={ep.still_path} alt=""/>:<span/>}<div><strong>{episodeCode(ep)} · {ep.name}</strong><small>{ep.overview||'No overview'}</small></div></div>
                          <span>{ep.air_date||'—'}</span>
                          <span className={ep.has_file?'episode-available':'episode-missing'}>{ep.has_file?'✓ Available':'Wanted'}</span>
                          <select value={ep.quality_profile_id??''} onChange={e=>{
                            if(e.target.value)void changeEpisode(item,ep,{quality_profile_id:Number(e.target.value)})
                            else void changeEpisode(item,ep,{clear_quality_profile:true})
                          }}>
                            <option value="">Inherit: {ep.effective_quality_profile_name||'None'}</option>
                            {profiles.map(p=><option key={p.id} value={p.id}>{p.name}</option>)}
                          </select>
                          <label className="episode-monitor"><input type="checkbox" checked={ep.monitored} disabled={busy===`episode-${ep.id}`} onChange={e=>void changeEpisode(item,ep,{monitored:e.target.checked})}/></label>
                          <button className="ghost-button" onClick={()=>setSearchTarget({
                            mediaId:item.id,tmdbId:item.tmdb_id,
                            title:`${item.name} ${episodeCode(ep)} · ${ep.name}`,
                            query:`${item.name} ${episodeCode(ep)}`,
                            profileId:ep.effective_quality_profile_id,
                            seasonNumber:ep.season_number,episodeNumber:ep.episode_number,isSeasonPack:false
                          })}>Search</button>
                        </div>)}
                      </div>}
                    </article>
                  })}
                </div>
              </>}
            </section>}
          </div>
        })}
        {!visible.length&&<div className="empty-state card">No series match the current filters.</div>}
      </div>

      <aside className="lib-side">
        <section className="lib-side-card">
          <header><strong>Library Overview</strong></header>
          <div className="overview-line"><i className="green"/><span>Monitored</span><b>{monitored}</b><em>{items.length?Math.round(monitored/items.length*100):0}%</em></div>
          <div className="overview-line"><i className="cyan"/><span>Available</span><b>{available}</b><em>{items.length?Math.round(available/items.length*100):0}%</em></div>
          <div className="overview-line"><i className="yellow"/><span>Downloading</span><b>{downloading}</b><em>{items.length?Math.round(downloading/items.length*100):0}%</em></div>
        </section>
        <section className="lib-side-card">
          <header><strong>Recently Added</strong></header>
          <div className="recent-poster-row">{recent.map(item=><button key={item.id} onClick={()=>void toggleExpanded(item)}>{item.poster_path?<img src={item.poster_path} alt=""/>:<span/>}<small>{item.name}</small></button>)}</div>
        </section>
        <section className="lib-side-card">
          <header><strong>Quality Profiles</strong></header>
          {profiles.slice(0,6).map(p=><div className="profile-line" key={p.id}><span>{p.name}</span><b>{items.filter(x=>x.quality_profile_id===p.id).length}</b></div>)}
        </section>
      </aside>
    </div>

    <ReleaseSearch
      open={!!searchTarget}
      onClose={()=>setSearchTarget(null)}
      mediaId={searchTarget?.mediaId??0}
      title={searchTarget?.title??''}
      tmdbId={searchTarget?.tmdbId??0}
      mediaType="series"
      profileId={searchTarget?.profileId??null}
      queryOverride={searchTarget?.query}
      seasonNumber={searchTarget?.seasonNumber}
      episodeNumber={searchTarget?.episodeNumber}
      isSeasonPack={searchTarget?.isSeasonPack??false}
    />
  </div>
}

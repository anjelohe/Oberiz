import { t, useTranslation, tf, portalLocale } from '../lib/i18n'
import { useEffect, useMemo, useState } from 'react'
import { Icon } from '../components/Icon'
import {
  getAutomationStatus, getDownloadsLive, getLibrarySummary, getMovies, getSeries, getSettings,
  getCalendar, getRssStatus, LibrarySummary, Movie, Series, Settings, TorrentLive, CalendarEpisode,
  AutomationStatus, RssStatus
} from '../lib/api'

type Indexers={total:number;enabled:number;invalid:number}
type History={id:number;event_type:string;title:string;detail:string|null;level:string;created_at:string}
type QbTest={status:string;version:string;host:string;latency_ms:number;auth_method:string}

function fmtBytes(bytes:number){
  if(!bytes)return '0 B'
  const units=['B','KB','MB','GB','TB'];let n=bytes,i=0
  while(n>=1024&&i<units.length-1){n/=1024;i++}
  return `${n.toFixed(i>=3?1:0)} ${units[i]}`
}
function fmtSpeed(bytes:number){return `${fmtBytes(bytes)}/s`}
function eta(sec:number){
  if(!Number.isFinite(sec)||sec<=0||sec>8640000)return '∞'
  if(sec<60)return `${Math.round(sec)}s`
  const m=Math.round(sec/60);if(m<60)return tf("{0}m left", m)
  return tf("{0}h {1}m left", Math.floor(m/60), m%60)
}
function relative(value:string){
  const d=new Date(value.replace(' ','T')+'Z').getTime()
  if(!Number.isFinite(d))return value
  const m=Math.max(0,Math.round((Date.now()-d)/60000))
  if(m<2)return t("just now"); if(m<60)return tf("{0} minutes ago", m)
  const h=Math.round(m/60); if(h<24)return tf("{0} hours ago", h)
  return tf("{0} days ago", Math.round(h/24))
}
function normalize(v:string){return v.toLowerCase().replace(/[^a-z0-9]+/g,' ').trim()}
function posterFor(name:string,movies:Movie[],series:Series[]){
  const n=normalize(name)
  const movie=movies.find(m=>n.includes(normalize(m.title))||normalize(m.title).includes(n))
  if(movie?.poster_path)return movie.poster_path
  const show=series.find(s=>n.includes(normalize(s.name))||normalize(s.name).includes(n))
  return show?.poster_path||null
}
function mediaLabel(name:string,movies:Movie[],series:Series[]){
  const n=normalize(name)
  if(series.some(s=>n.includes(normalize(s.name))))return t("Series")
  if(movies.some(m=>n.includes(normalize(m.title))))return t("Movie")
  return t("Media")
}
function qualityLabel(name:string){
  const u=name.toUpperCase()
  if(u.includes('2160P')||u.includes('4K'))return '4K'
  if(u.includes('1080P'))return '1080p'
  if(u.includes('720P'))return '720p'
  return ''
}
function dayLabel(date:string){
  const d=new Date(`${date}T12:00:00`)
  if(!Number.isFinite(d.getTime()))return date
  return d.toLocaleDateString(portalLocale(),{month:'short',day:'numeric'})
}
function MetricBars({value,label}:{value:number;label:string}){
  useTranslation()
  const percent=Math.round(Math.max(0,Math.min(100,value)))
  return <div className="stat-meter" title={label} aria-label={label}>
    <span className={percent>0?'on':''}/><span className={percent>25?'on':''}/><span className={percent>50?'on':''}/><span className={percent>75?'on':''}/><small>{label}</small>
  </div>
}
function shortActivity(row:History){
  if(row.event_type.includes('grabbed'))return t("Sent to qBittorrent")
  if(row.event_type.includes('missing'))return t("Torrent removed from qBittorrent")
  if(row.event_type.includes('import'))return row.level==='error'?t("Import needs attention"):t("Imported to library")
  if(row.event_type.includes('rss'))return row.level==='error'?t("RSS synchronization failed"):t("RSS synchronization completed")
  if(row.event_type.includes('automation'))return row.level==='error'?t("Automation needs attention"):t("Automation checked media")
  return row.detail?.split('·')[0]?.trim()||row.event_type.replace(/[._-]+/g,' ')
}
function episodeCode(episode:CalendarEpisode){return `S${String(episode.season_number).padStart(2,'0')}E${String(episode.episode_number).padStart(2,'0')}`}

export function Dashboard(){
  useTranslation()
  const [movies,setMovies]=useState<Movie[]>([])
  const [series,setSeries]=useState<Series[]>([])
  const [downloads,setDownloads]=useState<TorrentLive[]>([])
  const [indexers,setIndexers]=useState<Indexers>({total:0,enabled:0,invalid:0})
  const [history,setHistory]=useState<History[]>([])
  const [calendar,setCalendar]=useState<CalendarEpisode[]>([])
  const [storage,setStorage]=useState<LibrarySummary>({movie_files:0,series_files:0,movie_bytes:0,series_bytes:0,total_bytes:0,available_episodes:0,missing_aired_episodes:0,future_episodes:0})
  const [settings,setSettings]=useState<Settings|null>(null)
  const [qb,setQb]=useState<QbTest|null>(null)
  const [automation,setAutomation]=useState<AutomationStatus|null>(null)
  const [rss,setRss]=useState<RssStatus|null>(null)
  // Queue Health and indexer coverage used to fall back to "everything is
  // fine" (100%, "online") whenever these fetches failed, because an empty
  // `downloads`/`indexers` default is indistinguishable from a genuinely
  // empty queue/config. Tracking the fetch outcome itself separately from
  // the data lets those cards say "unknown" instead of fabricating health.
  const [downloadsFailed,setDownloadsFailed]=useState(false)
  const [indexersFailed,setIndexersFailed]=useState(false)

  async function load(){
    await Promise.all([
      getMovies().then(setMovies).catch(()=>{}),
      getSeries().then(setSeries).catch(()=>{}),
      getDownloadsLive().then(x=>{setDownloads(x.torrents??[]);setDownloadsFailed(false)}).catch(()=>setDownloadsFailed(true)),
      fetch('/api/indexers').then(r=>r.ok?r.json():Promise.reject()).then(x=>{setIndexers(x);setIndexersFailed(false)}).catch(()=>setIndexersFailed(true)),
      fetch('/api/history?limit=12').then(r=>r.ok?r.json():Promise.reject()).then(setHistory).catch(()=>{}),
      getCalendar(30,true).then(setCalendar).catch(()=>{}),
      getLibrarySummary().then(setStorage).catch(()=>{}),
      getSettings().then(setSettings).catch(()=>{}),
      fetch('/api/settings/test-qbittorrent',{method:'POST'}).then(r=>r.ok?r.json():Promise.reject()).then(setQb).catch(()=>setQb(null)),
      getAutomationStatus().then(setAutomation).catch(()=>{}),
      getRssStatus().then(setRss).catch(()=>{}),
    ])
  }

  useEffect(()=>{
    void load()
    const timer=setInterval(()=>void getDownloadsLive().then(x=>{setDownloads(x.torrents??[]);setDownloadsFailed(false)}).catch(()=>setDownloadsFailed(true)),5000)
    return()=>clearInterval(timer)
  },[])

  const monitoredMovies=movies.filter(x=>x.monitored).length
  const monitoredSeries=series.filter(x=>x.monitored).length
  const seriesCoverage=series.length?monitoredSeries/series.length*100:0
  const active=downloads.filter(x=>x.progress<.999999&&!/stopped|paused/i.test(x.state))
  const activeSpeed=downloads.reduce((a,x)=>a+(x.dlspeed||0),0)
  const activeBytes=active.reduce((a,x)=>a+(x.total_size||x.size||0),0)
  const completed=downloads.filter(x=>x.progress>=.999999)
  const queueHealth=downloadsFailed?null:downloads.length?Math.round(downloads.filter(x=>!/error|missing/i.test(x.state)).length/downloads.length*100):100
  const monitoredCoverage=movies.length?monitoredMovies/movies.length*100:0
  const downloadProgress=active.length?active.reduce((sum,item)=>sum+item.progress,0)/active.length*100:0
  const indexerCoverage=indexers.total?indexers.enabled/indexers.total*100:0
  const rssErrors=rss?.states.filter(x=>Boolean(x.last_error)||x.last_status==='error')??[]
  const automationErrors=automation?.states.filter(x=>Boolean(x.last_error)||x.status==='error')??[]

  const recentMedia=useMemo(()=>{
    const all=[
      ...movies.map(m=>({type:'movie' as const,id:m.id,title:m.title,year:m.year,poster:m.poster_path,created:m.created_at,wanted:m.monitored&&!m.available,available:m.available})),
      ...series.map(s=>({type:'series' as const,id:s.id,title:s.name,year:s.year,poster:s.poster_path,created:s.created_at,wanted:s.monitored&&s.missing_episode_count>0,available:s.available_episode_count>0})),
    ]
    return all.sort((a,b)=>b.created.localeCompare(a.created)).slice(0,5)
  },[movies,series])

  const upcoming=useMemo(()=>calendar.filter(x=>new Date(`${x.air_date}T23:59:59`).getTime()>=Date.now()).slice(0,6),[calendar])

  return <div className="dashboard-ref">
    <section className="dashboard-ref-stats dashboard-library-stats">
      <article className="dashboard-ref-stat"><span className="stat-orb movie"><Icon name="movies" size={26}/></span><div><small>{t("Monitored Movies")}</small><strong>{monitoredMovies}</strong><em>{movies.length} {t("total in library")}</em></div><MetricBars value={monitoredCoverage} label={tf("{0}% covered", Math.round(monitoredCoverage))}/></article>
      <article className="dashboard-ref-stat dashboard-series-stat"><span className="stat-orb movie"><Icon name="series" size={26}/></span><div><div className="series-stats-values"><small>{t("Monitored Series")}</small><small>{t("Episodes")}</small><strong>{monitoredSeries}</strong><strong>{storage.available_episodes}</strong><em>{series.length} {t("total in library")}</em><em title={tf("{0} aired missing · {1} future",storage.missing_aired_episodes,storage.future_episodes)}>{tf("{0} missing · {1} future",storage.missing_aired_episodes,storage.future_episodes)}</em></div></div><MetricBars value={seriesCoverage} label={tf("{0}% covered",Math.round(seriesCoverage))}/></article>
      <article className="dashboard-ref-stat"><span className="stat-orb download"><Icon name="downloads" size={27}/></span><div><small>{t("Active Downloads")}</small><strong>{active.length}</strong><em>{active.length?tf("{0} · {1} queued", fmtSpeed(activeSpeed), fmtBytes(activeBytes)):t("No active transfers")}</em></div><MetricBars value={downloadProgress} label={active.length?tf("{0}% average", Math.round(downloadProgress)):t("Idle")}/></article>
      <article className="dashboard-ref-stat"><span className="stat-orb indexer"><Icon name="indexers" size={27}/></span><div><small>{t("Active Indexers")}</small><strong>{indexersFailed?'—':`${indexers.enabled} / ${indexers.total}`}</strong><em>{indexersFailed?t("Could not load indexers"):indexers.invalid?t("Needs attention"):tf("{0} enabled (not a live health check)", indexers.enabled)}</em></div><MetricBars value={indexersFailed?0:indexerCoverage} label={indexersFailed?t("Unknown"):tf("{0} / {1} enabled", indexers.enabled, indexers.total)}/></article>
      <article className="dashboard-ref-stat"><span className="stat-orb health">♡</span><div><small>{t("Queue Health")}</small><strong>{queueHealth===null?'—':`${queueHealth}%`}</strong><em>{queueHealth===null?t("Could not reach qBittorrent"):queueHealth===100?t("Everything looks good"):t("Check download queue")}</em></div><MetricBars value={queueHealth??0} label={queueHealth===null?t("Unknown"):tf("{0}% healthy", queueHealth)}/></article>
    </section>

    <section className="dashboard-ref-grid">
      <div className="dashboard-ref-left">
        <section className="dashboard-ref-card dashboard-ref-downloads">
          <header><div><Icon name="downloads"/><h3>{t("Recent Downloads")}</h3></div><button onClick={()=>window.location.hash='/downloads'}>{t("View All")}</button></header>
          <div className="dashboard-ref-download-list">
            {downloads.slice(0,3).map((d,i)=>{
              const poster=posterFor(d.name,movies,series)
              return <article key={d.hash}>
                <div className="rd-poster">{poster?<img src={poster} alt=""/>:<span>{d.name.charAt(0).toUpperCase()}</span>}</div>
                <div className="rd-main">
                  <strong>{d.name}</strong>
                  <small>{mediaLabel(d.name,movies,series)} {qualityLabel(d.name)&&`· ${qualityLabel(d.name)}`} · {fmtBytes(d.total_size||d.size)}</small>
                  <div className="rd-progress"><i style={{width:`${Math.round(d.progress*100)}%`}}/></div>
                </div>
                <div className="rd-status">
                  <b className={d.progress>=.999999?'complete':'downloading'}>{d.progress>=.999999?t("Completed"):t("Downloading")}</b>
                  <strong>{Math.round(d.progress*100)}%</strong>
                  <small>{d.progress>=.999999?t("Finished"):`${fmtSpeed(d.dlspeed)} · ${eta(d.eta)}`}</small>
                </div>
              </article>
            })}
            {!downloads.length&&<div className="dashboard-empty dashboard-empty-rich"><Icon name="downloads" size={24}/><strong>{t("Your download queue is clear")}</strong><span>{t("New grabs will appear here with their live progress.")}</span></div>}
          </div>
        </section>

        <div className="dashboard-ref-lower-grid">
          <section className="dashboard-ref-card dashboard-ref-activity">
            <header><div><Icon name="history"/><h3>{t("Recent Activity")}</h3></div><button onClick={()=>window.location.hash='/history'}>{t("View All")}</button></header>
            <div>
              {history.slice(0,6).map((row,i)=><article key={row.id}>
                <span className={`activity-icon a${i%4}`}>{i%4===0?'↓':i%4===1?'✓':i%4===2?'⌕':'◉'}</span>
                <strong>{row.title}</strong><span>{shortActivity(row)}</span><time>{relative(row.created_at)}</time>
              </article>)}
              {!history.length&&<div className="dashboard-empty dashboard-empty-rich"><Icon name="history" size={24}/><strong>{t("No recent activity")}</strong><span>{t("Imports, searches and downloads will be recorded here.")}</span></div>}
            </div>
          </section>

          <section className="dashboard-ref-card dashboard-ref-upcoming">
            <header><div><Icon name="calendar"/><h3>{t("Upcoming Episodes")}</h3></div><button onClick={()=>window.location.hash='/calendar'}>{t("View Calendar")}</button></header>
            <div className="upcoming-list">
              {upcoming.map(episode=><article key={`${episode.series_id}-${episode.season_number}-${episode.episode_number}`}><span>{dayLabel(episode.air_date)}</span><div><strong>{episode.series_name}</strong><small>{episodeCode(episode)} · {episode.episode_name}</small></div></article>)}
              {!upcoming.length&&<div className="dashboard-empty"><strong>{t("No upcoming episodes")}</strong><span>{t("Future monitored episodes will appear here.")}</span></div>}
            </div>
          </section>
        </div>
      </div>

      <div className="dashboard-ref-right">
        <section className="dashboard-ref-card dashboard-ref-recent">
          <header><div><Icon name="movies"/><h3>{t("Recently Added / Wanted")}</h3></div><button onClick={()=>window.location.hash='/movies'}>{t("View All")}</button></header>
          <div className="recent-poster-row">
            {recentMedia.map(item=><button key={`${item.type}-${item.id}`} onClick={()=>window.location.hash=item.type==='movie'?'/movies':'/series'}>
              <div>{item.poster?<img src={item.poster} alt=""/>:<span>{item.title.charAt(0)}</span>}<b className={item.wanted?'wanted':item.available?'available':"monitored"}>{item.wanted?t("Wanted"):item.available?t("Available"):t("Monitored")}</b></div>
              <strong>{item.title}</strong><small>{item.year||'—'}</small>
            </button>)}
            {!recentMedia.length&&<div className="dashboard-empty dashboard-empty-rich"><Icon name="plus" size={25}/><strong>{t("Build your library")}</strong><span>{t("Add a movie or series to start tracking wanted media here.")}</span><button onClick={()=>window.dispatchEvent(new Event('oberiz-open-add-media'))}>{t("Add Media")}</button></div>}
          </div>
        </section>

        <section className="dashboard-ref-card dashboard-ref-system">
          <header><div><span className="system-heart">♡</span><h3>{t("System Status")}</h3></div><button onClick={()=>window.location.hash='/settings'}>{t("Settings")}</button></header>
          <div className="system-services">
            <article><span className="service-badge tmdb">TMDB</span><div><strong>TMDB</strong><b className={settings?.tmdb_api_key_set?'ok':'bad'}>● {settings?.tmdb_api_key_set?t("Configured"):t("Not configured")}</b><small>{t("Metadata, posters and media information")}</small></div></article>
            <article><span className="service-badge tvdb">TVDB</span><div><strong>TVDB</strong><b className={settings?.tvdb_api_key_set?'ok':'bad'}>● {settings?.tvdb_api_key_set?t("Configured"):t("Not configured")}</b><small>{t("Series identifiers for episode matching")}</small></div></article>
            <article><span className="service-badge qb">qb</span><div><strong>qBittorrent</strong><b className={qb?'ok':'bad'}>● {qb?t("Connected"):t("Unavailable")}</b><small>{qb?`${qb.version} · ${qb.latency_ms} ms`:t("Check Settings connection")}</small></div></article>
          </div>
          <div className="system-mini-grid">
            <article title={t("Size of indexed media files, not disk capacity or all torrents.")}><span>◉</span><div><small>{t("Indexed Storage")}</small><strong>{fmtBytes(storage.total_bytes)}</strong><em>{storage.movie_files+storage.series_files} {t("files")}</em></div></article>
            <article><span>⚙</span><div><small>{t("Automation")}</small><strong>{automation?.running?t("RUNNING"):automation?.enabled?t("ON"):t("OFF")}</strong><em>{automation?.running?tf("{0}/{1} processed", automation.completed_items, automation.total_items):automationErrors.length?tf("{0} historical issue{1}", automationErrors.length, automationErrors.length===1?'':'s'):automation?tf("every {0} min", automation.interval_minutes):t("status unavailable")}</em></div></article>
            <article><span>◌</span><div><small>RSS</small><strong>{rss?.enabled?t("ON"):t("OFF")}</strong><em>{rss?.configured_feeds?tf("{0} feed{1} · {2}", rss.configured_feeds, rss.configured_feeds===1?'':'s', rssErrors.length?t("needs review"):t('healthy')):t("No feeds configured")}</em></div></article>
            <article><span>▦</span><div><small>{t("Upcoming")}</small><strong>{upcoming.length}</strong><em>{upcoming[0]?`${upcoming[0].series_name} · ${dayLabel(upcoming[0].air_date)}`:t("No upcoming episodes")}</em></div></article>
          </div>
        </section>
      </div>
    </section>
  </div>
}

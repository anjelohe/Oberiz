import { useEffect, useMemo, useState } from 'react'
import { CalendarEpisode, getCalendar } from '../lib/api'
import { Icon } from '../components/Icon'

function code(row:CalendarEpisode){return `S${String(row.season_number).padStart(2,'0')}E${String(row.episode_number).padStart(2,'0')}`}
// Local calendar date, not UTC: toISOString() reports the UTC date, which
// disagrees with "today" as the viewer's own clock sees it for roughly a
// third of the day (more, the further their timezone sits from UTC) — right
// around midnight local time this used to show yesterday's or tomorrow's
// date as "Today" depending on which side of UTC midnight the browser's
// timezone put it. The backend separately computes its own day boundary
// from the server's localtime (see calendar.rs/series.rs), so the two still
// won't always agree when server and browser sit in different timezones —
// there's no single shared "today" across a network boundary — but the
// frontend's own date should at least match what's on the viewer's screen.
function today(){
  const d=new Date()
  const year=d.getFullYear()
  const month=String(d.getMonth()+1).padStart(2,'0')
  const day=String(d.getDate()).padStart(2,'0')
  return `${year}-${month}-${day}`
}
function labelDate(value:string){
  const d=new Date(`${value}T12:00:00`)
  const now=today()
  if(value===now)return 'Today'
  return d.toLocaleDateString(undefined,{weekday:'long',day:'numeric',month:'short'})
}

export function Calendar(){
  const [rows,setRows]=useState<CalendarEpisode[]>([])
  const [days,setDays]=useState(45)
  const [onlyMonitored,setOnlyMonitored]=useState(true)
  const [error,setError]=useState('')
  async function load(){try{setRows(await getCalendar(days,true));setError('')}catch(e){setError(e instanceof Error?e.message:String(e))}}
  useEffect(()=>{void load()},[days])
  const filtered=useMemo(()=>rows.filter(r=>!onlyMonitored||r.monitored),[rows,onlyMonitored])
  const groups=useMemo(()=>{
    const map=new Map<string,CalendarEpisode[]>()
    filtered.forEach(r=>map.set(r.air_date,[...(map.get(r.air_date)||[]),r]))
    return [...map.entries()]
  },[filtered])
  const upcoming=filtered.filter(r=>r.air_date>=today()&&!r.has_file).length
  const available=filtered.filter(r=>r.has_file).length

  return <div className="calendar-page">
    <section className="page-heading calendar-heading">
      <div><span className="eyebrow">SERIES</span><h1>Calendar</h1><p>Upcoming and recently aired episodes from your Oberiz library.</p></div>
      <div className="calendar-controls">
        <select value={days} onChange={e=>setDays(Number(e.target.value))}><option value={14}>14 days</option><option value={30}>30 days</option><option value={45}>45 days</option><option value={90}>90 days</option></select>
        <label><input type="checkbox" checked={onlyMonitored} onChange={e=>setOnlyMonitored(e.target.checked)}/> Monitored only</label>
        <button className="ghost-button" onClick={()=>void load()}>Refresh</button>
      </div>
    </section>
    <section className="calendar-stats">
      <div className="card"><Icon name="series"/><span>Episodes in range</span><strong>{filtered.length}</strong></div>
      <div className="card"><span className="calendar-dot upcoming"/><span>Upcoming missing</span><strong>{upcoming}</strong></div>
      <div className="card"><span className="calendar-dot available"/><span>Available</span><strong>{available}</strong></div>
    </section>
    {error&&<div className="error-box">{error}</div>}
    <section className="calendar-list">
      {groups.map(([date,items])=><div className="calendar-day" key={date}>
        <header><strong>{labelDate(date)}</strong><span>{date}</span><b>{items.length}</b></header>
        <div>{items.map(item=><article className="calendar-episode" key={`${item.series_id}-${item.season_number}-${item.episode_number}`}>
          {item.poster_path?<img src={item.poster_path} alt=""/>:<div className="calendar-poster-empty"><Icon name="series"/></div>}
          <div className="calendar-copy"><small>{item.series_name}</small><h3>{code(item)} · {item.episode_name}</h3><span>{item.monitored?'● Monitored':'○ Not monitored'}</span></div>
          <strong className={item.has_file?'calendar-state available':'calendar-state wanted'}>{item.has_file?'✓ Available':item.air_date<today()?'! Missing':'Upcoming'}</strong>
        </article>)}</div>
      </div>)}
      {!groups.length&&<div className="empty-state card">No episodes in this calendar window.</div>}
    </section>
  </div>
}

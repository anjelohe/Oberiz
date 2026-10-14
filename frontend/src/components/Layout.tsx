import { ReactNode, useEffect, useRef, useState } from 'react'
import { Icon } from './Icon'
import { AddMediaModal } from './AddMediaModal'
import { useTranslation, t, tf } from '../lib/i18n'
import { Diagnostics, Movie, Series, getDiagnostics, getMovies, getSeries } from '../lib/api'

export type Page = 'dashboard'|'movies'|'series'|'profiles'|'downloads'|'imports'|'indexers'|'history'|'calendar'|'settings'

const items:Array<{id:Page;label:string;icon:string}>=[
  {id:'dashboard',label:'Dashboard',icon:'dashboard'},
  {id:'indexers',label:'Indexers',icon:'indexers'},
  {id:'downloads',label:'Downloads',icon:'downloads'},
  {id:'movies',label:'Movies',icon:'movies'},
  {id:'series',label:'Series',icon:'series'},
  {id:'profiles',label:'Profiles',icon:'profiles'},
]

export function Layout({
  page,onPage,children,theme,onTheme
}:{
  page:Page;onPage:(p:Page)=>void;children:ReactNode;
  theme:'dark'|'middle'|'light';onTheme:(theme:'dark'|'middle'|'light')=>void
}){
  const [open,setOpen]=useState(false)
  useTranslation()
  const [collapsed,setCollapsed]=useState(()=>localStorage.getItem('oberiz.sidebar-collapsed')==='true')
  const [addMediaOpen,setAddMediaOpen]=useState(false)
  const [query,setQuery]=useState('')
  const [movies,setMovies]=useState<Movie[]>([])
  const [series,setSeries]=useState<Series[]>([])
  const [diagnostics,setDiagnostics]=useState<Diagnostics|null>(null)
  const [diagnosticsFailed,setDiagnosticsFailed]=useState(false)
  const searchRef=useRef<HTMLInputElement>(null)

  useEffect(()=>{
    const handler=()=>setAddMediaOpen(true)
    window.addEventListener('oberiz-open-add-media',handler)
    return()=>window.removeEventListener('oberiz-open-add-media',handler)
  },[])
  useEffect(()=>{
    function refresh(){void Promise.all([getMovies(),getSeries()]).then(([movieRows,seriesRows])=>{setMovies(movieRows);setSeries(seriesRows)}).catch(()=>{})}
    refresh()
    // Global search used to only load this once on mount: adding or
    // deleting a title left it invisible/still-listed in search results
    // until a full page reload, long after the rest of the app had moved on.
    window.addEventListener('oberiz-library-changed',refresh)
    return()=>window.removeEventListener('oberiz-library-changed',refresh)
  },[])
  useEffect(()=>{
    // The sidebar badge used to just say "All Systems Operational" as fixed
    // text regardless of what was actually happening — it couldn't tell a
    // healthy install from a dead database or qBittorrent apart. This polls
    // real diagnostics and distinguishes "didn't check yet"/"check itself
    // failed" from an actually-confirmed problem, instead of defaulting to
    // a green claim in both cases.
    function refresh(){
      getDiagnostics().then(d=>{setDiagnostics(d);setDiagnosticsFailed(false)}).catch(()=>setDiagnosticsFailed(true))
    }
    refresh()
    const timer=window.setInterval(refresh,30000)
    return()=>window.clearInterval(timer)
  },[])
  const systemHealthy=diagnostics!==null&&diagnostics.database==='ok'&&diagnostics.recent_errors.length===0
  const systemStatus=diagnosticsFailed?'unknown':diagnostics===null?'unknown':systemHealthy?'ok':'warn'
  const systemLabel=systemStatus==='unknown'?t("Status unknown"):systemStatus==='warn'?t("Needs attention"):t("All Systems Operational")
  useEffect(()=>{
    const shortcut=(event:KeyboardEvent)=>{if((event.ctrlKey||event.metaKey)&&event.key.toLowerCase()==='k'){event.preventDefault();searchRef.current?.focus()}}
    window.addEventListener('keydown',shortcut)
    return()=>window.removeEventListener('keydown',shortcut)
  },[])

  function navigate(p:Page){onPage(p);setOpen(false)}
  const needle=query.trim().toLowerCase()
  const results=needle?[...movies.filter(movie=>movie.title.toLowerCase().includes(needle)).map(movie=>({type:"movies" as Page,title:movie.title,meta:tf("Movie · {0}", movie.year||'—')})),...series.filter(show=>show.name.toLowerCase().includes(needle)).map(show=>({type:'series' as Page,title:show.name,meta:tf("Series · {0}", show.year||'—')}))].slice(0,8):[]

  function toggleSidebar(){setCollapsed(current=>{const next=!current;localStorage.setItem('oberiz.sidebar-collapsed',String(next));return next})}

  return <div className={`app-shell ${collapsed?'sidebar-collapsed':''}`}>
    {open&&<button className="mobile-backdrop" aria-label={t("Close menu")} onClick={()=>setOpen(false)}/>}
    <div className="topbar-brand">
      <button className="sidebar-collapse" type="button" onClick={toggleSidebar} title={collapsed?t("Expand menu"):t("Collapse menu")} aria-label={collapsed?t("Expand menu"):t("Collapse menu")}>☰</button>
      <button className="topbar-logo" onClick={()=>navigate('dashboard')} aria-label={t("Go to dashboard")}><img src={theme==='middle'?'/oberiz-logo-middle.png':'/oberiz-logo.png'} alt="Oberiz"/></button>
    </div>
    <aside className={`sidebar ${open?'mobile-open':''}`}>
      <nav>{items.map(item=><button key={item.id} className={`nav-item ${page===item.id?'active':''}`} onClick={()=>navigate(item.id)} aria-label={t(item.label)} aria-current={page===item.id?'page':undefined}><Icon name={item.icon}/><span>{t(item.label)}</span></button>)}</nav>
      <div className="sidebar-spacer"/>
      <label className="sidebar-theme">
        <span>{t('Theme')}</span>
        <select value={theme} onChange={e=>onTheme(e.target.value==='light'||e.target.value==='middle'?e.target.value:'dark')}>
          <option value="dark">{t('Dark')}</option>
          <option value="middle">{t('Middle')}</option>
          <option value="light">{t('Light')}</option>
        </select>
      </label>
      <div className="sidebar-theme-icons" role="group" aria-label={t("Theme")}><button className={theme==='dark'?'active':''} type="button" onClick={()=>onTheme('dark')} title={t("Dark theme")} aria-label={t("Dark theme")} aria-pressed={theme==='dark'}>◐</button><button className={theme==='middle'?'active':''} type="button" onClick={()=>onTheme('middle')} title={t("Middle theme")} aria-label={t("Middle theme")} aria-pressed={theme==='middle'}>●</button><button className={theme==='light'?'active':''} type="button" onClick={()=>onTheme('light')} title={t("Light theme")} aria-label={t("Light theme")} aria-pressed={theme==='light'}>☀</button></div>
      <button className={`system-ok ${systemStatus==='warn'?'warn':systemStatus==='unknown'?'unknown':''}`} onClick={()=>navigate('settings')} title={t("Open system diagnostics")}><span/> {t(systemLabel)}</button>
      <div className="version"><b>v1.1.1</b><span>{t("Self-hosted Media Automation")}</span></div>
    </aside>

    <main className="main">
      <header className="topbar">
        <button className="mobile-menu" aria-label={t("Open menu")} onClick={()=>setOpen(true)}>☰</button>
        <div className="global-search-wrap"><div className="global-search"><Icon name="search" size={20}/><input ref={searchRef} value={query} onChange={event=>setQuery(event.target.value)} onKeyDown={event=>{if(event.key==='Escape')setQuery('')}} placeholder={t('Search Oberiz...')}/><kbd>Ctrl K</kbd></div>{needle&&<div className="global-search-results">{results.map(result=><button key={`${result.type}-${result.title}`} onClick={()=>{navigate(result.type);setQuery('')}}><strong>{result.title}</strong><small>{result.meta}</small></button>)}{!results.length&&<p>{t("No movies or series in your library match “")}{query}”.</p>}</div>}</div>
        <div className="top-actions">
          <button className="primary-button top-add" onClick={()=>setAddMediaOpen(true)}><Icon name="plus" size={18}/>{t('Add Media')}</button>
          <button className="top-icon-button" aria-label={t("Open imports")} title={t('Imports & Reseed')} onClick={()=>navigate('imports')}><Icon name="imports" size={19}/></button>
          <button className="top-icon-button" aria-label={t("Open history")} title={t('History')} onClick={()=>navigate('history')}><Icon name="history" size={19}/></button>
          <button className="top-icon-button" aria-label={t("Open calendar")} title={t('Calendar')} onClick={()=>navigate('calendar')}><Icon name="calendar" size={19}/></button>
          <button className="top-icon-button" aria-label={t("Open notifications")} title={t('Notifications & activity')} onClick={()=>navigate('history')}><Icon name="bell" size={19}/></button>
          <button className="top-icon-button" aria-label={t("Open settings")} title={t('Settings')} onClick={()=>navigate('settings')}><Icon name="settings" size={19}/></button>
        </div>
      </header>
      <div className="content">{children}</div>
    </main>
    <AddMediaModal open={addMediaOpen} onClose={()=>setAddMediaOpen(false)} onAdded={type=>navigate(type==='movie'?"movies":'series')}/>
  </div>
}

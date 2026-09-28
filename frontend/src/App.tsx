import './pages/Downloads.css'
import './pages/Indexers.css'
import './pages/Profiles.css'
import { useEffect, useState } from 'react'
import { Layout, Page } from './components/Layout'
import { Dashboard } from './pages/Dashboard'
import { Movies } from './pages/Movies'
import { Series } from './pages/Series'
import { Profiles } from './pages/Profiles'
import { Downloads } from './pages/Downloads'
import { Imports } from './pages/Imports'
import { Indexers } from './pages/Indexers'
import { History } from './pages/History'
import { Settings } from './pages/Settings'
import { Calendar } from './pages/Calendar'
import { Login } from './components/Login'
import { Setup } from './components/Setup'
import { AuthStatus, getAuthStatus, getSettings, saveSettings } from './lib/api'

const components: Record<Page, React.ReactNode> = {
  dashboard: <Dashboard />,
  movies: <Movies />,
  series: <Series />,
  profiles: <Profiles />,
  downloads: <Downloads />,
  imports: <Imports />,
  indexers: <Indexers />,
  history: <History />,
  calendar: <Calendar />,
  settings: <Settings />,
}

function pageFromHash(): Page {
  const candidate = window.location.hash.replace('#/', '') as Page
  return candidate in components ? candidate : 'dashboard'
}

type Theme = 'dark'|'middle'|'light'

function normalizeTheme(value:string|null|undefined):Theme{
  return value==='light'||value==='middle'?value:'dark'
}

function initialTheme():Theme{
  return normalizeTheme(localStorage.getItem('oberiz.theme'))
}

function savedTheme():Theme|undefined{
  const saved=localStorage.getItem('oberiz.theme')
  return saved==='dark'||saved==='middle'||saved==='light'?saved:undefined
}

export default function App() {
  const [page,setPage]=useState<Page>(pageFromHash())
  const [theme,setTheme]=useState<Theme>(initialTheme())
  const [authChecked,setAuthChecked]=useState(false)
  const [authStatus,setAuthStatus]=useState<AuthStatus|null>(null)

  function refreshAuthStatus(){
    return getAuthStatus().then(setAuthStatus).catch(()=>{}).finally(()=>setAuthChecked(true))
  }

  useEffect(()=>{
    refreshAuthStatus()
    const onAuthRequired=()=>{void refreshAuthStatus()}
    window.addEventListener('oberiz-auth-required',onAuthRequired)
    return()=>window.removeEventListener('oberiz-auth-required',onAuthRequired)
  },[])

  useEffect(()=>{
    document.documentElement.dataset.theme=theme
  },[theme])

  useEffect(()=>{
    getSettings().then(settings=>{
      // A local choice is applied instantly and must not be overwritten by an
      // older backend process while it is being restarted or upgraded.
      const next=savedTheme()??normalizeTheme(settings.ui_theme)
      setTheme(next)
      localStorage.setItem('oberiz.theme',next)
    }).catch(()=>{})
    const event=(ev:Event)=>{
      const next=normalizeTheme((ev as CustomEvent<string>).detail)
      setTheme(next)
    }
    window.addEventListener('oberiz-theme-changed',event)
    return()=>window.removeEventListener('oberiz-theme-changed',event)
  },[])

  useEffect(()=>{
    const handler=()=>setPage(pageFromHash())
    window.addEventListener('hashchange',handler)
    return()=>window.removeEventListener('hashchange',handler)
  },[])

  function navigate(next:Page){
    window.location.hash=`/${next}`
    setPage(next)
  }

  async function changeTheme(next:Theme){
    setTheme(next)
    localStorage.setItem('oberiz.theme',next)
    try{await saveSettings({ui_theme:next})}catch{/* local preference still applies until backend is available */}
  }

  if (!authChecked) return null
  if (!authStatus || !authStatus.enabled) return <Setup onSuccess={()=>void refreshAuthStatus()} />
  if (!authStatus.authenticated) return <Login onSuccess={()=>void refreshAuthStatus()} />

  return <Layout page={page} onPage={navigate} theme={theme} onTheme={changeTheme}>
    {components[page]}
  </Layout>
}

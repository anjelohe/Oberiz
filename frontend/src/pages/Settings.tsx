import { ChangeEvent, FormEvent, useEffect, useState } from 'react'
import {
  backupDownloadUrl, createBackup, deleteBackup, getAuthStatus, getAutomationStatus, getBackups, getDiagnostics, getRssStatus, getSeedPolicies, getSettings, logout, rescanLibrary, restoreBackup, runAutomationNow, runRssNow, saveSeedPolicy, setAdminPassword, uploadBackup,
  saveSettings, AuthStatus, BackupList, Diagnostics, RssStatus, SeedPolicy, Settings as SettingsType, testQBittorrent,
  getLibraryRoots, LibraryRootStatus
} from '../lib/api'
import { Icon } from '../components/Icon'
import { FolderPicker } from '../components/FolderPicker'
import { StorageSettings } from '../components/StorageSettings'
import { normalizeLanguage, setPortalLanguage, useTranslation, t, tf, portalLocale } from '../lib/i18n'

const blankPolicy:SeedPolicy={
  indexer_id:'',
  min_seed_time_minutes:0,
  min_ratio:0,
  requirement_mode:'manual',
  cleanup_mode:'manual',
  updated_at:'',
}

function formatBytes(bytes:number){
  if(bytes<1024) return `${bytes} B`
  if(bytes<1024*1024) return `${(bytes/1024).toFixed(1)} KB`
  return `${(bytes/(1024*1024)).toFixed(1)} MB`
}

function LibraryRootsEditor({title,paths,onChange}:{title:string;paths:string[];onChange:(paths:string[])=>void}){
  return <div className="library-roots-editor"><h4>{title}</h4>
    {paths.map((path,i)=><div className="library-root-row" key={i}><label>{t('Library folder')}<input value={path} onChange={e=>onChange(paths.map((p,j)=>j===i?e.target.value:p))}/><FolderPicker value={path} onChange={next=>onChange(paths.map((p,j)=>j===i?next:p))}/></label><button type="button" className="danger-button" onClick={()=>onChange(paths.filter((_,j)=>j!==i))}>{t('Remove')}</button></div>)}
    <FolderPicker value="" label={t('Add folders')} onChange={()=>{}} onSelectMany={selected=>onChange([...new Set([...paths,...selected])])}/>
    <button type="button" className="ghost-button" onClick={()=>onChange([...paths,''])}>{t('Add folder')}</button>
  </div>
}

export function Settings(){
  useTranslation()
  const [data,setData]=useState<SettingsType|null>(null)
  const [tmdb,setTmdb]=useState(''),[tvdb,setTvdb]=useState(''),[password,setPassword]=useState(''),[apiKey,setApiKey]=useState('')
  const [message,setMessage]=useState(''),[qbMessage,setQbMessage]=useState(''),[autoMessage,setAutoMessage]=useState(''),[rssMessage,setRssMessage]=useState('')
  const [autoRunning,setAutoRunning]=useState(false)
  const [policies,setPolicies]=useState<SeedPolicy[]>([])
  const [policy,setPolicy]=useState<SeedPolicy>(blankPolicy)
  const [policyMessage,setPolicyMessage]=useState('')
  const [scanMessage,setScanMessage]=useState('')
  const [rootStatus,setRootStatus]=useState<LibraryRootStatus[]>([])
  const [scanning,setScanning]=useState(false)
  const [apiMessage,setApiMessage]=useState('')
  const [apiKeyVisible,setApiKeyVisible]=useState(false)
  const [apiCopied,setApiCopied]=useState(false)
  const [diagnostics,setDiagnostics]=useState<Diagnostics|null>(null)
  const [diagnosticsMessage,setDiagnosticsMessage]=useState('')
  const [rssStatus,setRssStatus]=useState<RssStatus|null>(null)
  const [backups,setBackups]=useState<BackupList|null>(null)
  const [backupMessage,setBackupMessage]=useState('')
  const [authStatus,setAuthStatus]=useState<AuthStatus|null>(null)
  const [authCurrentPassword,setAuthCurrentPassword]=useState('')
  const [authNewPassword,setAuthNewPassword]=useState('')
  const [authConfirmPassword,setAuthConfirmPassword]=useState('')
  const [authMessage,setAuthMessage]=useState('')
  useEffect(()=>{
    getSettings().then(setData).catch(e=>setMessage(String(e)))
    getLibraryRoots().then(setRootStatus).catch(()=>{})
    getSeedPolicies().then(setPolicies).catch(()=>{})
    getRssStatus().then(setRssStatus).catch(()=>{})
    getBackups().then(setBackups).catch(e=>setBackupMessage(e instanceof Error?e.message:String(e)))
    getAuthStatus().then(setAuthStatus).catch(()=>{})
    const timer=window.setInterval(()=>getSettings().then(fresh=>setData(current=>current?{...current,last_external_report:fresh.last_external_report}:fresh)).catch(()=>{}),15000)
    return()=>window.clearInterval(timer)
  },[])
  useEffect(()=>{
    if(!autoRunning)return
    const timer=window.setInterval(()=>{getAutomationStatus().then(status=>{
      if(status.running) setAutoMessage(tf("Running · {0}/{1} · {2}", status.completed_items, status.total_items, status.current_item||t('Preparing next search…')))
    }).catch(()=>{})},700)
    return()=>window.clearInterval(timer)
  },[autoRunning])
  if(!data)return <div className="empty-state card">{t("Loading settings…")} {message}</div>

  async function submit(e:FormEvent){
    e.preventDefault();setMessage(t("Saving…"))
    try{
      await saveSettings({
        tmdb_api_key:tmdb===''?null:tmdb,
        tvdb_api_key:tvdb===''?null:tvdb,
        qbittorrent_host:data!.qbittorrent_host,
        qbittorrent_port:data!.qbittorrent_port,
        qbittorrent_username:data!.qbittorrent_username,
        qbittorrent_password:password===''?null:password,
        qbittorrent_https:data!.qbittorrent_https,
        movie_library_paths:data!.movie_library_paths,
        series_library_paths:data!.series_library_paths,
        library_source:data!.library_source,
        pause_on_stale_inventory:data!.pause_on_stale_inventory,
        library_scan_enabled:data!.library_scan_enabled,
        library_scan_interval_minutes:data!.library_scan_interval_minutes,
        custom_indexers_path:data!.custom_indexers_path,
        upstream_indexers_path:data!.upstream_indexers_path,
        automation_enabled:data!.automation_enabled,
        automation_interval_minutes:data!.automation_interval_minutes,
        rss_enabled:data!.rss_enabled,
        rss_interval_minutes:data!.rss_interval_minutes,
        backup_enabled:data!.backup_enabled,
        backup_interval_hours:data!.backup_interval_hours,
        backup_retention_count:data!.backup_retention_count,
        qbittorrent_rename_enabled:data!.rename_enabled,
        movie_naming_template:data!.movie_naming_template,
        series_naming_template:data!.series_naming_template,
        keep_reseed_metadata:data!.keep_reseed_metadata,
        cleanup_after_seed:data!.cleanup_after_seed,
        ui_theme:data!.ui_theme,
        ui_language:data!.ui_language,
        api_enabled:data!.api_enabled,
        api_key:apiKey===''?null:apiKey,
        overseerr_compat_enabled:data!.overseerr_compat_enabled,
      })
      localStorage.setItem('oberiz.theme',data!.ui_theme)
      setPortalLanguage(data!.ui_language)
      document.documentElement.dataset.theme=data!.ui_theme
      window.dispatchEvent(new CustomEvent('oberiz-theme-changed',{detail:data!.ui_theme}))
      setTmdb('');setTvdb('');setPassword('');
      const savedKey=apiKey!==''
      const saved=await getSettings()
      setData(saved);setMessage(t("Saved"))
      setRootStatus(await getLibraryRoots())
      if(savedKey) setApiMessage(t("API key saved. Copy it now — it will not be shown again."))
    }catch(e){setMessage(e instanceof Error?e.message:String(e))}
  }

  async function testQb(){
    try{
      await saveSettings({
        qbittorrent_host:data!.qbittorrent_host,
        qbittorrent_port:data!.qbittorrent_port,
        qbittorrent_username:data!.qbittorrent_username,
        qbittorrent_password:password===''?null:password,
        qbittorrent_https:data!.qbittorrent_https,
      })
      const r=await testQBittorrent()
      setQbMessage(tf("Connected · qBittorrent {0} · {1} ms · {2}", r.version, r.latency_ms, r.auth_method))
      setPassword('')
      // Only the qBittorrent fields were actually persisted above — replacing
      // the whole draft with a fresh getSettings() here used to silently
      // discard any other unsaved edit (movies_path, import_method, etc.)
      // the user had made in this same form but not saved yet.
      const fresh=await getSettings()
      setData(current=>current?{
        ...current,
        qbittorrent_host:fresh.qbittorrent_host,
        qbittorrent_port:fresh.qbittorrent_port,
        qbittorrent_username:fresh.qbittorrent_username,
        qbittorrent_password_set:fresh.qbittorrent_password_set,
        qbittorrent_https:fresh.qbittorrent_https,
      }:fresh)
    }catch(e){setQbMessage(e instanceof Error?e.message:String(e))}
  }

  async function savePolicy(){
    if(!policy.indexer_id.trim()){setPolicyMessage(t("Indexer ID is required"));return}
    try{
      setPolicyMessage(t("Saving…"))
      await saveSeedPolicy(policy.indexer_id,{
        min_seed_time_minutes:policy.min_seed_time_minutes,
        min_ratio:policy.min_ratio,
        requirement_mode:policy.requirement_mode,
        cleanup_mode:policy.cleanup_mode,
      })
      setPolicies(await getSeedPolicies())
      setPolicy(blankPolicy)
      setPolicyMessage(t("Saved"))
    }catch(e){setPolicyMessage(e instanceof Error?e.message:String(e))}
  }

  async function scan(mediaType:'movie'|'series'|'all'){
    setScanning(true)
    try{
      setScanMessage(t("Scanning library…"))
      await saveSettings({movie_library_paths:data!.movie_library_paths,series_library_paths:data!.series_library_paths,library_source:data!.library_source,library_scan_enabled:data!.library_scan_enabled,library_scan_interval_minutes:data!.library_scan_interval_minutes})
      const rows=await rescanLibrary(mediaType)
      setScanMessage(rows.length?rows.map(r=>`${r.root_path}: ${r.error||tf("{0}: {1}/{2} matched",t(r.media_type==='movie'?'Movies':'Series'),r.matched_files,r.scanned_files)}`).join(' · '):t("Configure folders and scan"))
      if(mediaType==='all'){window.dispatchEvent(new CustomEvent('oberiz-library-changed',{detail:'movie'}));window.dispatchEvent(new CustomEvent('oberiz-library-changed',{detail:'series'}))}
      else window.dispatchEvent(new CustomEvent('oberiz-library-changed',{detail:mediaType}))
    }catch(e){setScanMessage(e instanceof Error?e.message:String(e))}
    finally{setScanning(false);getLibraryRoots().then(setRootStatus).catch(()=>{})}
  }

  function generateApiKey(){
    const bytes=new Uint8Array(24);crypto.getRandomValues(bytes)
    const key=`obrz_${Array.from(bytes).map(v=>v.toString(16).padStart(2,'0')).join('')}`
    setApiKey(key);setApiKeyVisible(true);setApiCopied(false)
    setApiMessage(data?.api_key_set?t("New key generated. Save Settings to replace the current key."):t("New key generated. Save Settings to activate it."))
  }

  async function copyApiKey(){
    if(!apiKey)return
    try{
      await navigator.clipboard.writeText(apiKey)
      setApiCopied(true);setApiMessage(t("API key copied to clipboard."))
    }catch{
      setApiMessage(t("Could not access clipboard. Select and copy the key manually."))
    }
  }

  async function saveAdminPassword(){
    if(authNewPassword.length<8){setAuthMessage(t("Password must be at least 8 characters"));return}
    if(authNewPassword!==authConfirmPassword){setAuthMessage(t("Passwords do not match"));return}
    try{
      setAuthMessage(t("Saving…"))
      await setAdminPassword(authStatus?.enabled?authCurrentPassword:null,authNewPassword)
      setAuthCurrentPassword('');setAuthNewPassword('');setAuthConfirmPassword('')
      setAuthStatus(await getAuthStatus())
      setAuthMessage(t("Admin password saved. Use it next time Oberiz asks you to sign in."))
    }catch(e){setAuthMessage(e instanceof Error?e.message:String(e))}
  }

  async function removeAdminPassword(){
    if(!window.confirm(t("Remove the admin password? Oberiz will ask you to create a new one the next time it's opened; there is no way to leave it unprotected.")))return
    try{
      setAuthMessage(t("Removing…"))
      await setAdminPassword(authCurrentPassword,null)
      setAuthCurrentPassword('');setAuthNewPassword('');setAuthConfirmPassword('')
      setAuthStatus(await getAuthStatus())
      setAuthMessage(t("Admin password removed."))
    }catch(e){setAuthMessage(e instanceof Error?e.message:String(e))}
  }

  async function loadDiagnostics(){
    try { setDiagnosticsMessage(t("Collecting diagnostic data…")); setDiagnostics(await getDiagnostics()); setDiagnosticsMessage(t("Updated")) }
    catch(e) { setDiagnosticsMessage(e instanceof Error?e.message:String(e)) }
  }

  async function makeBackup(){
    try { setBackupMessage(t("Creating backup…")); const result=await createBackup(); setBackups(await getBackups()); setBackupMessage(`${t(result.message)} ${formatBytes(result.backup.size_bytes)}`) }
    catch(e) { setBackupMessage(e instanceof Error?e.message:String(e)) }
  }

  async function restore(filename:string){
    if(!window.confirm(tf("Restore {0}? This replaces the current Oberiz database with that backup. Current media, requests, settings and history will be reverted.", filename))) return
    try { setBackupMessage(t("Restoring backup…")); const result=await restoreBackup(filename); setBackups(await getBackups()); setBackupMessage(tf("{0} Reloading…", t(result.message))); window.setTimeout(()=>window.location.reload(),700) }
    catch(e) { setBackupMessage(e instanceof Error?e.message:String(e)) }
  }

  async function removeBackup(filename:string){
    if(!window.confirm(tf("Delete local backup {0}? This cannot be undone.", filename))) return
    try { setBackupMessage(t("Deleting backup…")); const result=await deleteBackup(filename); setBackups(await getBackups()); setBackupMessage(t(result.message)) }
    catch(e) { setBackupMessage(e instanceof Error?e.message:String(e)) }
  }

  async function upload(e:ChangeEvent<HTMLInputElement>){
    const file=e.target.files?.[0]; e.target.value=''; if(!file)return
    try { setBackupMessage(t("Uploading backup…")); const result=await uploadBackup(file); setBackups(await getBackups()); setBackupMessage(`${t(result.message)} ${formatBytes(result.backup.size_bytes)}`) }
    catch(error) { setBackupMessage(error instanceof Error?error.message:String(error)) }
  }

  return <>
    <div className="page-heading"><div><h1><Icon name="settings" size={32}/> {t('Settings')}</h1><p>{t('Services, automation, importing, seeding and appearance.')}</p></div></div>
    <form className="settings-layout" onSubmit={submit}>
      <div className="settings-main">

<div className="settings-row settings-row-three" data-settings-row="1">
        <section className="settings-section card">
          <div className="settings-title"><div><h3>{t('Appearance')}</h3><span>{t('Stored permanently in Oberiz settings.')}</span></div></div>
          <label>{t('Portal language')}
            <select value={data.ui_language || 'en'} onChange={e=>{
              const language=normalizeLanguage(e.target.value)
              setData({...data,ui_language:language})
              setPortalLanguage(language)
            }}>
              <option value="en">English</option>
              <option value="es">Español</option>
              <option value="fr">Français</option>
              <option value="de">Deutsch</option>
              <option value="it">Italiano</option>
              <option value="pt">Português</option>
            </select>
          </label>
          <label>{t('Theme')}
            <select value={data.ui_theme} onChange={e=>{
              const theme=e.target.value==='light'||e.target.value==='middle'?e.target.value:'dark'
              setData({...data,ui_theme:theme})
              document.documentElement.dataset.theme=theme
            }}>
              <option value="dark">{t('Dark')}</option>
              <option value="middle">{t('Middle')}</option>
              <option value="light">{t('Light')}</option>
            </select>
          </label>
        </section>

        <section className="settings-section card support-section settings-wide">
          <div className="settings-title"><div><h3>{t("Support Oberiz")}</h3><span>{t("Oberiz is free to use. Donations are entirely optional.")}</span></div></div>
          <div className="support-content">
            <img src="/paypal-donate-qr.png" alt={t("PayPal donation QR code for Oberiz")} />
            <div>
              <p>{t("If Oberiz is useful to you, you can support its development through PayPal.")}</p>
              <small>{t("TMDB is configured independently by each user with their own API credential and subject to TMDB’s terms.")}</small>
            </div>
            <a className="primary-button support-button" href="https://paypal.me/anjelohe" target="_blank" rel="noreferrer">{t("Support via PayPal")}</a>
          </div>
        </section>

        <section className="settings-section card">
          <div className="settings-title"><div><h3>{t("Diagnostics")}</h3><span>{t("Read-only status for support and troubleshooting. Secrets are never included.")}</span></div></div>
          <div className="settings-actions"><span>{diagnosticsMessage}</span><button type="button" className="ghost-button" onClick={()=>void loadDiagnostics()}>{t("Refresh diagnostics")}</button></div>
          {diagnostics&&<div className="settings-help">
            <p>Oberiz {diagnostics.version} · {diagnostics.operating_system} {t("· Database:")} {diagnostics.database}</p>
            <p>TMDB: {diagnostics.tmdb_configured?t('configured'):t("not configured")} · qBittorrent: {diagnostics.qbittorrent_configured?t('configured'):t("not configured")} {t("· Indexers:")} {diagnostics.enabled_indexers}/{diagnostics.total_indexers} {t("enabled · Automation:")} {diagnostics.automation_enabled?"enabled" :t('disabled')}</p>
            {diagnostics.recent_errors.length>0&&<p>{t("Recent errors:")} {diagnostics.recent_errors.map(error=>`${error.source}: ${error.message}`).join(' · ')}</p>}
            {diagnostics.recent_errors.length===0&&<p>{t("No recent errors recorded.")}</p>}
          </div>}
        </section>
        </div>

        <div className="settings-row settings-row-three" data-settings-row="2">
        <section className="settings-section card settings-wide">
          <div className="settings-title"><div><h3>{t("RSS Automation")}</h3><span>{t("Process only new feed releases from configured indexers; each release is deduplicated before matching.")}</span></div><span className="connection">{data.rss_enabled?t("● Enabled"):t("○ Disabled")}</span></div>
          <div className="form-grid automation-grid">
            <label className="toggle-row"><input type="checkbox" checked={data.rss_enabled} onChange={e=>setData({...data,rss_enabled:e.target.checked})}/> {t("Enable RSS feed sync")}</label>
            <label>{t("Interval (minutes)")}<input type="number" min="5" max="1440" value={data.rss_interval_minutes} onChange={e=>setData({...data,rss_interval_minutes:Number(e.target.value)})}/></label>
          </div>
          <div className="settings-actions"><span>{rssMessage}</span><button type="button" className="ghost-button" onClick={async()=>{try{await saveSettings({rss_enabled:data.rss_enabled,rss_interval_minutes:data.rss_interval_minutes});const [run,status]=await Promise.all([runRssNow(),getRssStatus()]);setRssStatus(status);setRssMessage(tf("RSS run · {0} feeds · {1} new · {2} matched · {3} grabbed · {4} errors · {5} configured", run.feeds, run.new_items, run.matched, run.grabbed, run.errors, status.configured_feeds)) }catch(e){setRssMessage(e instanceof Error?e.message:String(e))}}}>{t("Sync Now")}</button></div>
          <small className="settings-help">{t("Set an optional")} <b>{t("RSS feed URL")}</b> {t("in each Indexer configuration. Existing releases are remembered, so a feed item is evaluated once only.")}</small>
          {rssStatus&&<div className="settings-help"><p><b>{rssStatus.configured_feeds}</b> {t("configured feeds · last interval:")} {rssStatus.interval_minutes} min</p>{rssStatus.states.slice(0,5).map(item=><p key={item.indexer_id}><b>{item.indexer_id}</b> · {item.last_status} · {item.last_new_items} {t("new ·")} {item.last_matched_items} {t("matched ·")} {item.last_grabbed_items} {t("grabbed")}{item.last_error?` · ${item.last_error}`:''}</p>)}{rssStatus.states.length===0&&<p>{t("No RSS synchronization has run yet.")}</p>}</div>}
        </section>

        <section className="settings-section card settings-wide">
          <div className="settings-title"><div className="service-logo qb">qb</div><div><h3>qBittorrent</h3><span>{t("Download client")}</span></div></div>
          <div className="form-grid">
            <label>{t("Host")}<input value={data.qbittorrent_host} onChange={e=>setData({...data,qbittorrent_host:e.target.value})}/></label>
            <label>{t("Port")}<input type="number" value={data.qbittorrent_port} onChange={e=>setData({...data,qbittorrent_port:Number(e.target.value)})}/></label>
            <label>{t("Username")}<input value={data.qbittorrent_username} onChange={e=>setData({...data,qbittorrent_username:e.target.value})}/></label>
            <label>{t("Password")}<input type="password" value={password} onChange={e=>setPassword(e.target.value)} placeholder={data.qbittorrent_password_set?t("Keep current if blank"):t("Password")}/></label>
          </div>
          <label className="toggle-row"><input type="checkbox" checked={data.qbittorrent_https} onChange={e=>setData({...data,qbittorrent_https:e.target.checked})}/> {t("Use HTTPS")}</label>
          <small className="settings-help">{t("Set download destinations in qBittorrent categories. Oberiz does not need NAS credentials.")}</small>
          <div className="settings-actions"><span>{qbMessage}</span><button type="button" className="ghost-button" onClick={()=>void testQb()}>{t("Test Connection")}</button></div>
        </section>

        <section className="settings-section card settings-full">
          <div className="settings-title"><div><h3>{t("Seed Policies")}</h3><span>{t("Per-indexer cleanup rules. No policy means no automatic deletion.")}</span></div></div>
          {policies.length>0&&<div className="seed-policy-list">
            {policies.map(p=><button type="button" className="seed-policy-row" key={p.indexer_id} onClick={()=>setPolicy(p)}>
              <strong>{p.indexer_id}</strong>
              <span>{(p.min_seed_time_minutes/1440).toFixed(1)} {t("days · ratio")} {p.min_ratio.toFixed(2)} · {p.requirement_mode}</span>
              <em>{p.cleanup_mode}</em>
            </button>)}
          </div>}
          <div className="seed-policy-editor">
            <label>{t("Indexer ID")}<input value={policy.indexer_id} onChange={e=>setPolicy({...policy,indexer_id:e.target.value})} placeholder={t("e.g. my-private-tracker")}/></label>
            <label>{t("Minimum seed days")}<input type="number" min="0" step=".25" value={policy.min_seed_time_minutes/1440} onChange={e=>setPolicy({...policy,min_seed_time_minutes:Math.round(Number(e.target.value)*1440)})}/></label>
            <label>{t("Minimum ratio")}<input type="number" min="0" step=".1" value={policy.min_ratio} onChange={e=>setPolicy({...policy,min_ratio:Number(e.target.value)})}/></label>
            <label>{t("Requirement")}
              <select value={policy.requirement_mode} onChange={e=>setPolicy({...policy,requirement_mode:e.target.value as SeedPolicy['requirement_mode']})}>
                <option value="manual">{t("Manual")}</option><option value="time">{t("Time")}</option><option value="ratio">Ratio</option><option value="either">{t("Either")}</option><option value="both">{t("Both")}</option>
              </select>
            </label>
            <label>{t("Cleanup")}
              <select value={policy.cleanup_mode} onChange={e=>setPolicy({...policy,cleanup_mode:e.target.value as SeedPolicy['cleanup_mode']})}>
                <option value="manual">{t("Manual")}</option><option value="never">{t("Never auto-delete")}</option>
                <option value="remove_torrent_keep_files">{t("Remove torrent, keep original download")}</option>
              </select>
            </label>
            <button type="button" className="ghost-button" onClick={()=>void savePolicy()}>{t("Save Policy")}</button>
          </div>
          <div className="settings-actions"><span>{policyMessage}</span></div>
        </section>
        </div>

        <div className="settings-row settings-row-three" data-settings-row="3">
        <section className="settings-section card">
          <div className="settings-title"><div className="service-logo">TMDB</div><div><h3>TMDB</h3><span>{t("Movies and series metadata")}</span></div><span className="connection">{data.tmdb_api_key_set?t("● Configured"):t("○ Not configured")}</span></div>
          <label>{t("Credential")}<input type="password" value={tmdb} onChange={e=>setTmdb(e.target.value)} placeholder={data.tmdb_api_key_set?t("Leave blank to keep current"):t("API key or Read Access Token")}/></label>
        </section>

        <section className="settings-section card">
          <div className="settings-title"><div className="service-logo">TVDB</div><div><h3>TVDB</h3><span>{t("Optional series identifier lookup for compatible indexers")}</span></div><span className="connection">{data.tvdb_api_key_set?t("● Configured"):t("○ Not configured")}</span></div>
          <label>{t("API key")}<input type="password" value={tvdb} onChange={e=>setTvdb(e.target.value)} placeholder={data.tvdb_api_key_set?t("Leave blank to keep current"):'TVDB API key'}/></label>
          <small className="settings-help">{t("Used only to resolve a TVDB series ID for indexers that request it. It is never sent to indexers.")}</small>
        </section>

        <section className="settings-section card">
          <div className="settings-title"><div><h3>{t("Automation")}</h3><span>{t("Automatically search monitored media using its assigned profile.")}</span></div><span className="connection">{data.automation_enabled?t("● Enabled"):t("○ Disabled")}</span></div>
          <div className="form-grid automation-grid">
            <label className="toggle-row"><input type="checkbox" checked={data.automation_enabled} onChange={e=>setData({...data,automation_enabled:e.target.checked})}/> {t("Enable automatic searches")}</label>
            <label>{t("Interval (minutes)")}<input type="number" min="5" max="1440" value={data.automation_interval_minutes} onChange={e=>setData({...data,automation_interval_minutes:Number(e.target.value)})}/></label>
          </div>
          <div className="settings-actions"><span>{autoMessage}</span><button type="button" className="ghost-button" disabled={autoRunning} onClick={async()=>{try{setAutoRunning(true);setAutoMessage(t("Running automation and checking qBittorrent…"));await saveSettings({automation_enabled:data.automation_enabled,automation_interval_minutes:data.automation_interval_minutes});const r=await runAutomationNow();const status=await getAutomationStatus();setAutoMessage(tf("Completed · {0} missing · {1} searched · {2} grabbed · {3} skipped · {4} errors · {5} monitored", r.reconciled_missing, r.searched, r.grabbed, r.skipped, r.errors, status.monitored_movies+status.monitored_series)) }catch(e){setAutoMessage(e instanceof Error?e.message:String(e))}finally{setAutoRunning(false)}}}>{autoRunning?t("Running…"):t("Run Now")}</button></div>
        </section>
        </div>

        <div className="settings-row settings-row-full" data-settings-row="4">
        <section className="settings-section card settings-full">
          <div className="settings-title"><div><h3>{t("qBittorrent naming & recovery")}</h3><span>{t("Disabled by default. Oberiz requests changes through qBittorrent; it never opens media files.")}</span></div></div>
          <div className="settings-path-grid">
            <label className="toggle-row"><input type="checkbox" checked={data.rename_enabled} onChange={e=>setData({...data,rename_enabled:e.target.checked})}/>{t("Rename completed downloads through qBittorrent")}</label>
            <label className="toggle-row"><input type="checkbox" checked={data.keep_reseed_metadata} onChange={e=>setData({...data,keep_reseed_metadata:e.target.checked})}/>{t("Keep .torrent + mapping for future reseed")}</label>
            <label className="toggle-row"><input type="checkbox" checked={data.cleanup_after_seed} onChange={e=>setData({...data,cleanup_after_seed:e.target.checked})}/>{t("Apply sharing policies (keep all downloaded files)")}</label>
          </div>
          <label>{t("Movie naming template")}<input value={data.movie_naming_template} disabled={!data.rename_enabled} onChange={e=>setData({...data,movie_naming_template:e.target.value})}/></label>
          <label>{t("Series naming template")}<input value={data.series_naming_template} disabled={!data.rename_enabled} onChange={e=>setData({...data,series_naming_template:e.target.value})}/></label>
          {data.rename_enabled&&<p role="alert" className="settings-help">{t("qBittorrent does not provide atomic no-overwrite renaming. Visible destinations are checked; do not run other organizers on these folders simultaneously.")}</p>}
          <small className="settings-help">{t("Common tokens:")} {'{Title} {Year} {Resolution} {Source} {Codec} {HDR} {Audio} {Language}'}{t(". Series also supports")} {'{Season} {Season:00} {Episode} {Episode:00} {EpisodeTitle}'}</small>
          <p className="settings-help">{t("Only Oberiz downloads are processed. Ambiguous files, external subtitles and naming conflicts require manual review.")}</p>
        </section>
        </div>

        <div className="settings-row settings-row-full" data-settings-row="5">
        <StorageSettings data={data} setData={setData}
          existingLibrary={<div className="storage-existing-library">
          <div className="settings-path-grid">
            <LibraryRootsEditor title={t("Movies")} paths={data.movie_library_paths} onChange={movie_library_paths=>setData({...data,movie_library_paths})}/>
            <LibraryRootsEditor title={t("Series")} paths={data.series_library_paths} onChange={series_library_paths=>setData({...data,series_library_paths})}/>
          </div>
          <div className="library-maintenance">
          <div className="library-scan-controls">
            <label className="toggle-row"><input type="checkbox" checked={data.library_scan_enabled} onChange={e=>setData({...data,library_scan_enabled:e.target.checked})}/>{t("Schedule library scans")}</label>
            <label>{t("Interval (minutes)")}<input type="number" min="5" max="10080" value={data.library_scan_interval_minutes} onChange={e=>setData({...data,library_scan_interval_minutes:Number(e.target.value)})}/></label>
          <div className="settings-actions settings-scan-actions">
            <button type="button" disabled={scanning} className="ghost-button" onClick={()=>void scan('movie')}>{t("Rescan Movies")}</button>
            <button type="button" disabled={scanning} className="ghost-button" onClick={()=>void scan('series')}>{t("Rescan Series")}</button>
            <button type="button" disabled={scanning} className="primary-button" onClick={()=>void scan('all')}>{t("Rescan All")}</button>
          </div>
          </div>
          {scanMessage&&<small className="settings-help" role="status">{scanMessage}</small>}
          <div className="library-scan-help"><small className="settings-help">{t("Rescan does not move or rename files. It indexes files already present in the configured library paths, refreshes Movies availability and links SxxExx files to Series episodes.")}</small><small className="settings-help">{t("Unavailable disks preserve indexed files and hold automatic acquisition to avoid duplicate downloads.")}</small></div>
          {rootStatus.length>0&&<div className="library-scan-groups">{(['movie','series'] as const).map(kind=>{
            const roots=rootStatus.filter(root=>root.media_type===kind)
            return roots.length>0&&<section className="library-scan-group" key={kind}><header><Icon name={kind==='movie'?'movies':'series'} size={17}/><strong>{t(kind==='movie'?'Movies':'Series')}</strong><span>{roots.length}</span></header><div className="library-root-status-list">{roots.map(root=><div className="library-root-status" key={root.path}><strong title={root.path}>{root.path}</strong><span className={root.accessible?'accessible':'unavailable'}>{root.accessible?t("Accessible"):t("Unavailable")}</span><small>{root.last_scan_at?new Date(root.last_scan_at.replace(' ','T')+'Z').toLocaleString(portalLocale()):t("Not scanned yet")}</small>{root.error&&<small role="alert">{root.error}</small>}</div>)}</div></section>
          })}</div>}
          </div>
        </div>}/>
        </div>

        <div className="settings-row settings-row-two" data-settings-row="7">
        <section className="settings-section card indexer-folders-section">
          <div className="settings-title"><div><h3>{t("Indexer folders")}</h3><span>{t("Where tracker definitions are read from. Normally there is no need to change them.")}</span></div></div>
          <label>{t("Custom indexers")}<input value={data.custom_indexers_path} onChange={e=>setData({...data,custom_indexers_path:e.target.value})}/><FolderPicker value={data.custom_indexers_path} onChange={custom_indexers_path=>setData({...data,custom_indexers_path})}/><small className="settings-help">{t("Your private tracker definitions (.yml). They are never published or synchronized.")}</small></label>
          <label>{t("Upstream indexers")}<input value={data.upstream_indexers_path} onChange={e=>setData({...data,upstream_indexers_path:e.target.value})}/><FolderPicker value={data.upstream_indexers_path} onChange={upstream_indexers_path=>setData({...data,upstream_indexers_path})}/><small className="settings-help">{t("Public definitions Oberiz downloads when you synchronize the catalogue.")}</small></label>
        </section>

        <section className="settings-section card api-key-section settings-full">
          <div className="settings-title"><div><h3>{t("Public API v1")}</h3><span>{t("Direct integration for your multimedia app or other request clients.")}</span></div><span className="connection">{data.api_enabled?t("● Enabled"):t("○ Disabled")}</span></div>
          <label className="toggle-row"><input type="checkbox" checked={data.api_enabled} onChange={e=>setData({...data,api_enabled:e.target.checked})}/> {t("Enable public request API")}</label>

          <label>{t("API key")}
            <div className="api-key-field">
              <input
                type={apiKeyVisible?'text':'password'}
                value={apiKey}
                onChange={e=>{setApiKey(e.target.value);setApiCopied(false)}}
                placeholder={data.api_key_set?t("Set — generate a new key to replace it"):t("Generate or enter a key")}
                autoComplete="off"
              />
              <button type="button" className="ghost-button" disabled={!apiKey} onClick={()=>setApiKeyVisible(v=>!v)}>{apiKeyVisible?t("Hide"):t("Show")}</button>
              <button type="button" className="ghost-button" disabled={!apiKey} onClick={()=>void copyApiKey()}>{apiCopied?t("Copied"):t("Copy")}</button>
            </div>
          </label>

          {apiKey&&<div className="api-key-warning"><b>{t("Copy this key now.")}</b><span>{t("Oberiz stores only its hash, not the key itself — once you leave or reload this page it can't be shown again. Generate a new one if you lose it.")}</span></div>}
          <div className="settings-actions"><span>{apiMessage}</span><button type="button" className="ghost-button" onClick={generateApiKey}>{data.api_key_set?t("Regenerate key"):t("Generate key")}</button></div>
          <small className="settings-help">{t("Clients send")} <b>X-Api-Key</b>{t(". Endpoints: GET /api/v1/status, POST/GET /api/v1/requests, GET /api/v1/requests/:id.")}</small>
          <label className="toggle-row"><input type="checkbox" checked={data.overseerr_compat_enabled} onChange={e=>setData({...data,overseerr_compat_enabled:e.target.checked})}/> {t("Enable Overseerr compatibility")}</label>
          {data.overseerr_compat_enabled&&<div className="overseerr-guide"><b>{t("Configure Overseerr")}</b><span>{t("Add Oberiz twice under")} <strong>{t("Settings → Services")}</strong>{t(", using the API key above:")}</span><code>{t("Movies (Radarr): http://your-oberiz-host:2032/radarr")}</code><code>{t("Series (Sonarr): http://your-oberiz-host:2032/sonarr")}</code><small>{t("Use the same key in both services. The movie and series root folders and quality profiles come from Oberiz.")}</small></div>}
        </section>
        </div>

        <div className="settings-row settings-row-full" data-settings-row="8">
        <section className="settings-section card settings-full">
          <div className="settings-title"><div><h3>{t("Backup & Restore")}</h3><span>{t("Consistent SQLite snapshots, including data currently in WAL.")}</span></div></div>
          <div className="form-grid automation-grid">
            <label className="toggle-row"><input type="checkbox" checked={data.backup_enabled} onChange={e=>setData({...data,backup_enabled:e.target.checked})}/> {t("Schedule automatic backups")}</label>
            <label>{t("Interval (hours)")}<input type="number" min="1" max="720" value={data.backup_interval_hours} disabled={!data.backup_enabled} onChange={e=>setData({...data,backup_interval_hours:Number(e.target.value)})}/></label>
            <label>{t("Keep latest copies")}<input type="number" min="1" max="100" value={data.backup_retention_count} disabled={!data.backup_enabled} onChange={e=>setData({...data,backup_retention_count:Number(e.target.value)})}/></label>
          </div>
          <div className="settings-actions"><span>{backupMessage}</span><label className="ghost-button file-button">{t("Upload backup")}<input type="file" accept=".sqlite,.sqlite3,.db,application/vnd.sqlite3" onChange={e=>void upload(e)}/></label><button type="button" className="primary-button" onClick={()=>void makeBackup()}>{t("Create Backup")}</button></div>
          <small className="settings-help">{t("Backups are stored in")} <b>{backups?.directory||'backups'}</b>{t(". Restoring replaces all current database data; media files and qBittorrent are not changed.")}</small>
          {backups&&<div className="backup-list">
            {backups.backups.map(backup=><div className="backup-row" key={backup.filename}><div><strong>{backup.filename}</strong><span>{new Date(backup.created_at).toLocaleString(portalLocale())} · {formatBytes(backup.size_bytes)}</span></div><div className="backup-actions"><a className="ghost-button" href={backupDownloadUrl(backup.filename)}>{t("Download")}</a><button type="button" className="danger-button" onClick={()=>void removeBackup(backup.filename)}>{t("Delete")}</button><button type="button" className="danger-button" onClick={()=>void restore(backup.filename)}>{t("Restore")}</button></div></div>)}
            {backups.backups.length===0&&<p className="settings-help">{t("No backups created yet.")}</p>}
          </div>}
        </section>
        </div>

        <div className="settings-row settings-row-full" data-settings-row="9">
        <section className="settings-section card settings-full">
          <div className="settings-title"><div><h3>{t("Admin Password")}</h3><span>{t("Protects the whole Oberiz interface and API when it is reachable on your network.")}</span></div><span className="connection">{authStatus?.enabled?t("● Enabled"):t("○ Disabled")}</span></div>
          <div className="form-grid">
            {authStatus?.enabled&&<label>{t("Current password")}<input type="password" value={authCurrentPassword} onChange={e=>setAuthCurrentPassword(e.target.value)} autoComplete="current-password"/></label>}
            <label>{t("New password")}<input type="password" value={authNewPassword} onChange={e=>setAuthNewPassword(e.target.value)} placeholder={t("At least 8 characters")} autoComplete="new-password"/></label>
            <label>{t("Confirm new password")}<input type="password" value={authConfirmPassword} onChange={e=>setAuthConfirmPassword(e.target.value)} autoComplete="new-password"/></label>
          </div>
          <div className="settings-actions">
            <span>{authMessage}</span>
            {authStatus?.enabled&&<button type="button" className="ghost-button" onClick={()=>void logout().then(()=>window.location.reload())}>{t("Sign out")}</button>}
            {authStatus?.enabled&&<button type="button" className="danger-button" onClick={()=>void removeAdminPassword()}>{t("Remove password")}</button>}
            <button type="button" className="primary-button" disabled={!authNewPassword} onClick={()=>void saveAdminPassword()}>{authStatus?.enabled?t("Change password"):t("Set password")}</button>
          </div>
          <small className="settings-help">{t("Required to use Oberiz. It keeps you signed in on this browser for 30 days; changing or removing it needs the current password.")}</small>
        </section>
        </div>

        <div className="settings-actions"><span>{message}</span><button className="primary-button" type="submit">{t("Save Settings")}</button></div>
      </div>
    </form>
  </>
}

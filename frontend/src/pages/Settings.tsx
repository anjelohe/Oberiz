import { ChangeEvent, FormEvent, useEffect, useState } from 'react'
import {
  backupDownloadUrl, createBackup, deleteBackup, getAutomationStatus, getBackups, getDiagnostics, getRssStatus, getSeedPolicies, getSettings, rescanLibrary, restoreBackup, runAutomationNow, runRssNow, saveSeedPolicy, uploadBackup,
  saveSettings, BackupList, Diagnostics, RssStatus, SeedPolicy, Settings as SettingsType, testQBittorrent
} from '../lib/api'
import { Icon } from '../components/Icon'
import { FolderPicker } from '../components/FolderPicker'

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

export function Settings(){
  const [data,setData]=useState<SettingsType|null>(null)
  const [tmdb,setTmdb]=useState(''),[password,setPassword]=useState(''),[apiKey,setApiKey]=useState('')
  const [message,setMessage]=useState(''),[qbMessage,setQbMessage]=useState(''),[autoMessage,setAutoMessage]=useState(''),[rssMessage,setRssMessage]=useState('')
  const [autoRunning,setAutoRunning]=useState(false)
  const [policies,setPolicies]=useState<SeedPolicy[]>([])
  const [policy,setPolicy]=useState<SeedPolicy>(blankPolicy)
  const [policyMessage,setPolicyMessage]=useState('')
  const [scanMessage,setScanMessage]=useState('')
  const [apiMessage,setApiMessage]=useState('')
  const [apiKeyVisible,setApiKeyVisible]=useState(false)
  const [apiCopied,setApiCopied]=useState(false)
  const [diagnostics,setDiagnostics]=useState<Diagnostics|null>(null)
  const [diagnosticsMessage,setDiagnosticsMessage]=useState('')
  const [rssStatus,setRssStatus]=useState<RssStatus|null>(null)
  const [backups,setBackups]=useState<BackupList|null>(null)
  const [backupMessage,setBackupMessage]=useState('')

  useEffect(()=>{
    getSettings().then(next=>{setData(next);setApiKey(next.api_key||'')}).catch(e=>setMessage(String(e)))
    getSeedPolicies().then(setPolicies).catch(()=>{})
    getRssStatus().then(setRssStatus).catch(()=>{})
    getBackups().then(setBackups).catch(e=>setBackupMessage(e instanceof Error?e.message:String(e)))
  },[])
  useEffect(()=>{
    if(!autoRunning)return
    const timer=window.setInterval(()=>{getAutomationStatus().then(status=>{
      if(status.running) setAutoMessage(`Running · ${status.completed_items}/${status.total_items} · ${status.current_item||'Preparing next search…'}`)
    }).catch(()=>{})},700)
    return()=>window.clearInterval(timer)
  },[autoRunning])
  if(!data)return <div className="empty-state card">Loading settings… {message}</div>

  async function submit(e:FormEvent){
    e.preventDefault();setMessage('Saving…')
    try{
      await saveSettings({
        tmdb_api_key:tmdb===''?null:tmdb,
        qbittorrent_host:data!.qbittorrent_host,
        qbittorrent_port:data!.qbittorrent_port,
        qbittorrent_username:data!.qbittorrent_username,
        qbittorrent_password:password===''?null:password,
        qbittorrent_https:data!.qbittorrent_https,
        movies_path:data!.movies_path,
        series_path:data!.series_path,
        downloads_path:data!.downloads_path,
        reseed_path:data!.reseed_path,
        custom_indexers_path:data!.custom_indexers_path,
        upstream_indexers_path:data!.upstream_indexers_path,
        automation_enabled:data!.automation_enabled,
        automation_interval_minutes:data!.automation_interval_minutes,
        rss_enabled:data!.rss_enabled,
        rss_interval_minutes:data!.rss_interval_minutes,
        backup_enabled:data!.backup_enabled,
        backup_interval_hours:data!.backup_interval_hours,
        backup_retention_count:data!.backup_retention_count,
        import_enabled:data!.import_enabled,
        import_method:data!.import_method,
        rename_enabled:data!.rename_enabled,
        movie_naming_template:data!.movie_naming_template,
        series_naming_template:data!.series_naming_template,
        keep_reseed_metadata:data!.keep_reseed_metadata,
        cleanup_after_seed:data!.cleanup_after_seed,
        torrent_metadata_path:data!.torrent_metadata_path,
        ui_theme:data!.ui_theme,
        api_enabled:data!.api_enabled,
        api_key:apiKey===''?null:apiKey,
        overseerr_compat_enabled:data!.overseerr_compat_enabled,
      })
      localStorage.setItem('oberiz.theme',data!.ui_theme)
      document.documentElement.dataset.theme=data!.ui_theme
      window.dispatchEvent(new CustomEvent('oberiz-theme-changed',{detail:data!.ui_theme}))
      setTmdb('');setPassword('');
      const next=await getSettings();setData(next);setApiKey(next.api_key||'');setMessage('Saved')
      if(next.api_key) setApiMessage('API key saved.')
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
      setQbMessage(`Connected · qBittorrent ${r.version} · ${r.latency_ms} ms · ${r.auth_method}`)
      setPassword('');setData(await getSettings())
    }catch(e){setQbMessage(e instanceof Error?e.message:String(e))}
  }

  async function savePolicy(){
    if(!policy.indexer_id.trim()){setPolicyMessage('Indexer ID is required');return}
    try{
      setPolicyMessage('Saving…')
      await saveSeedPolicy(policy.indexer_id,{
        min_seed_time_minutes:policy.min_seed_time_minutes,
        min_ratio:policy.min_ratio,
        requirement_mode:policy.requirement_mode,
        cleanup_mode:policy.cleanup_mode,
      })
      setPolicies(await getSeedPolicies())
      setPolicy(blankPolicy)
      setPolicyMessage('Saved')
    }catch(e){setPolicyMessage(e instanceof Error?e.message:String(e))}
  }

  async function scan(mediaType:'movie'|'series'|'all'){
    try{
      setScanMessage('Scanning library…')
      const rows=await rescanLibrary(mediaType)
      setScanMessage(rows.map(r=>`${r.media_type}: ${r.matched_files}/${r.scanned_files} matched`).join(' · '))
      if(mediaType==='all'){window.dispatchEvent(new CustomEvent('oberiz-library-changed',{detail:'movie'}));window.dispatchEvent(new CustomEvent('oberiz-library-changed',{detail:'series'}))}
      else window.dispatchEvent(new CustomEvent('oberiz-library-changed',{detail:mediaType}))
    }catch(e){setScanMessage(e instanceof Error?e.message:String(e))}
  }

  function generateApiKey(){
    const bytes=new Uint8Array(24);crypto.getRandomValues(bytes)
    const key=`obrz_${Array.from(bytes).map(v=>v.toString(16).padStart(2,'0')).join('')}`
    setApiKey(key);setApiKeyVisible(true);setApiCopied(false)
    setApiMessage(data?.api_key_set?'New key generated. Save Settings to replace the current key.':'New key generated. Save Settings to activate it.')
  }

  async function copyApiKey(){
    if(!apiKey)return
    try{
      await navigator.clipboard.writeText(apiKey)
      setApiCopied(true);setApiMessage('API key copied to clipboard.')
    }catch{
      setApiMessage('Could not access clipboard. Select and copy the key manually.')
    }
  }

  async function loadDiagnostics(){
    try { setDiagnosticsMessage('Collecting diagnostic data…'); setDiagnostics(await getDiagnostics()); setDiagnosticsMessage('Updated') }
    catch(e) { setDiagnosticsMessage(e instanceof Error?e.message:String(e)) }
  }

  async function makeBackup(){
    try { setBackupMessage('Creating backup…'); const result=await createBackup(); setBackups(await getBackups()); setBackupMessage(`${result.message} ${formatBytes(result.backup.size_bytes)}`) }
    catch(e) { setBackupMessage(e instanceof Error?e.message:String(e)) }
  }

  async function restore(filename:string){
    if(!window.confirm(`Restore ${filename}? This replaces the current Oberiz database with that backup. Current media, requests, settings and history will be reverted.`)) return
    try { setBackupMessage('Restoring backup…'); const result=await restoreBackup(filename); setBackups(await getBackups()); setBackupMessage(`${result.message} Reloading…`); window.setTimeout(()=>window.location.reload(),700) }
    catch(e) { setBackupMessage(e instanceof Error?e.message:String(e)) }
  }

  async function removeBackup(filename:string){
    if(!window.confirm(`Delete local backup ${filename}? This cannot be undone.`)) return
    try { setBackupMessage('Deleting backup…'); const result=await deleteBackup(filename); setBackups(await getBackups()); setBackupMessage(result.message) }
    catch(e) { setBackupMessage(e instanceof Error?e.message:String(e)) }
  }

  async function upload(e:ChangeEvent<HTMLInputElement>){
    const file=e.target.files?.[0]; e.target.value=''; if(!file)return
    try { setBackupMessage('Uploading backup…'); const result=await uploadBackup(file); setBackups(await getBackups()); setBackupMessage(`${result.message} ${formatBytes(result.backup.size_bytes)}`) }
    catch(error) { setBackupMessage(error instanceof Error?error.message:String(error)) }
  }

  return <>
    <div className="page-heading"><div><h1><Icon name="settings" size={32}/> Settings</h1><p>Services, automation, importing, seeding and appearance.</p></div></div>
    <form className="settings-layout" onSubmit={submit}>
      <div className="settings-main">

        <section className="settings-section card">
          <div className="settings-title"><div><h3>Appearance</h3><span>Stored permanently in Oberiz settings.</span></div></div>
          <label>Theme
            <select value={data.ui_theme} onChange={e=>{
              const theme=e.target.value==='light'||e.target.value==='middle'?e.target.value:'dark'
              setData({...data,ui_theme:theme})
              document.documentElement.dataset.theme=theme
            }}>
              <option value="dark">Dark</option>
              <option value="middle">Middle</option>
              <option value="light">Light</option>
            </select>
          </label>
        </section>

        <section className="settings-section card support-section">
          <div className="settings-title"><div><h3>Support Oberiz</h3><span>Oberiz is free to use. Donations are entirely optional.</span></div></div>
          <div className="support-content">
            <img src="/paypal-donate-qr.png" alt="PayPal donation QR code for Oberiz" />
            <div>
              <p>If Oberiz is useful to you, you can support its development through PayPal.</p>
              <a className="primary-button support-button" href="https://paypal.me/anjelohe" target="_blank" rel="noreferrer">Support via PayPal</a>
              <small>TMDB is configured independently by each user with their own API credential and subject to TMDB’s terms.</small>
            </div>
          </div>
        </section>

        <section className="settings-section card">
          <div className="settings-title"><div><h3>RSS Automation</h3><span>Process only new feed releases from configured indexers; each release is deduplicated before matching.</span></div><span className="connection">{data.rss_enabled?'● Enabled':'○ Disabled'}</span></div>
          <div className="form-grid automation-grid">
            <label className="toggle-row"><input type="checkbox" checked={data.rss_enabled} onChange={e=>setData({...data,rss_enabled:e.target.checked})}/> Enable RSS feed sync</label>
            <label>Interval (minutes)<input type="number" min="5" max="1440" value={data.rss_interval_minutes} onChange={e=>setData({...data,rss_interval_minutes:Number(e.target.value)})}/></label>
          </div>
          <div className="settings-actions"><span>{rssMessage}</span><button type="button" className="ghost-button" onClick={async()=>{try{await saveSettings({rss_enabled:data.rss_enabled,rss_interval_minutes:data.rss_interval_minutes});const [run,status]=await Promise.all([runRssNow(),getRssStatus()]);setRssStatus(status);setRssMessage(`RSS run · ${run.feeds} feeds · ${run.new_items} new · ${run.matched} matched · ${run.grabbed} grabbed · ${run.errors} errors · ${status.configured_feeds} configured`) }catch(e){setRssMessage(e instanceof Error?e.message:String(e))}}}>Sync Now</button></div>
          <small className="settings-help">Set an optional <b>RSS feed URL</b> in each Indexer configuration. Existing releases are remembered, so a feed item is evaluated once only.</small>
          {rssStatus&&<div className="settings-help"><p><b>{rssStatus.configured_feeds}</b> configured feeds · last interval: {rssStatus.interval_minutes} min</p>{rssStatus.states.slice(0,5).map(item=><p key={item.indexer_id}><b>{item.indexer_id}</b> · {item.last_status} · {item.last_new_items} new · {item.last_matched_items} matched · {item.last_grabbed_items} grabbed{item.last_error?` · ${item.last_error}`:''}</p>)}{rssStatus.states.length===0&&<p>No RSS synchronization has run yet.</p>}</div>}
        </section>

        <section className="settings-section card">
          <div className="settings-title"><div className="service-logo">TMDB</div><div><h3>TMDB</h3><span>Movies and series metadata</span></div><span className="connection">{data.tmdb_api_key_set?'● Configured':'○ Not configured'}</span></div>
          <label>Credential<input type="password" value={tmdb} onChange={e=>setTmdb(e.target.value)} placeholder={data.tmdb_api_key_set?'Leave blank to keep current':'API key or Read Access Token'}/></label>
        </section>

        <section className="settings-section card">
          <div className="settings-title"><div className="service-logo qb">qb</div><div><h3>qBittorrent</h3><span>Download client</span></div></div>
          <div className="form-grid">
            <label>Host<input value={data.qbittorrent_host} onChange={e=>setData({...data,qbittorrent_host:e.target.value})}/></label>
            <label>Port<input type="number" value={data.qbittorrent_port} onChange={e=>setData({...data,qbittorrent_port:Number(e.target.value)})}/></label>
            <label>Username<input value={data.qbittorrent_username} onChange={e=>setData({...data,qbittorrent_username:e.target.value})}/></label>
            <label>Password<input type="password" value={password} onChange={e=>setPassword(e.target.value)} placeholder={data.qbittorrent_password_set?'Keep current if blank':'Password'}/></label>
          </div>
          <label className="toggle-row"><input type="checkbox" checked={data.qbittorrent_https} onChange={e=>setData({...data,qbittorrent_https:e.target.checked})}/> Use HTTPS</label>
          <div className="settings-actions"><span>{qbMessage}</span><button type="button" className="ghost-button" onClick={()=>void testQb()}>Test Connection</button></div>
        </section>

        <section className="settings-section card">
          <div className="settings-title"><div><h3>Automation</h3><span>Automatically search monitored media using its assigned profile.</span></div><span className="connection">{data.automation_enabled?'● Enabled':'○ Disabled'}</span></div>
          <div className="form-grid automation-grid">
            <label className="toggle-row"><input type="checkbox" checked={data.automation_enabled} onChange={e=>setData({...data,automation_enabled:e.target.checked})}/> Enable automatic searches</label>
            <label>Interval (minutes)<input type="number" min="5" max="1440" value={data.automation_interval_minutes} onChange={e=>setData({...data,automation_interval_minutes:Number(e.target.value)})}/></label>
          </div>
          <div className="settings-actions"><span>{autoMessage}</span><button type="button" className="ghost-button" disabled={autoRunning} onClick={async()=>{try{setAutoRunning(true);setAutoMessage('Running automation and checking qBittorrent…');await saveSettings({automation_enabled:data.automation_enabled,automation_interval_minutes:data.automation_interval_minutes});const r=await runAutomationNow();const status=await getAutomationStatus();setAutoMessage(`Completed · ${r.reconciled_missing} missing · ${r.searched} searched · ${r.grabbed} grabbed · ${r.skipped} skipped · ${r.errors} errors · ${status.monitored_movies+status.monitored_series} monitored`) }catch(e){setAutoMessage(e instanceof Error?e.message:String(e))}finally{setAutoRunning(false)}}}>{autoRunning?'Running…':'Run Now'}</button></div>
        </section>

        <section className="settings-section card">
          <div className="settings-title"><div><h3>Import & Naming</h3><span>Completed downloads are imported without breaking active seeding.</span></div><span className="connection">{data.import_enabled?'● Enabled':'○ Disabled'}</span></div>
          <div className="settings-path-grid">
            <label className="toggle-row"><input type="checkbox" checked={data.import_enabled} onChange={e=>setData({...data,import_enabled:e.target.checked})}/> Import completed downloads</label>
            <label>Import method
              <select value={data.import_method} onChange={e=>setData({...data,import_method:e.target.value})}>
                <option value="auto">Auto — hardlink, then copy fallback</option>
                <option value="hardlink">Hardlink only</option>
                <option value="copy">Copy</option>
                <option value="move">Move (only for setups without seeding)</option>
              </select>
            </label>
            <label className="toggle-row"><input type="checkbox" checked={data.rename_enabled} onChange={e=>setData({...data,rename_enabled:e.target.checked})}/> Rename imported files</label>
            <label className="toggle-row"><input type="checkbox" checked={data.keep_reseed_metadata} onChange={e=>setData({...data,keep_reseed_metadata:e.target.checked})}/> Keep .torrent + mapping for future reseed</label>
            <label className="toggle-row"><input type="checkbox" checked={data.cleanup_after_seed} onChange={e=>setData({...data,cleanup_after_seed:e.target.checked})}/> Cleanup downloads after seed policy is satisfied</label>
          </div>
          <label>Movie naming template<input value={data.movie_naming_template} disabled={!data.rename_enabled} onChange={e=>setData({...data,movie_naming_template:e.target.value})}/></label>
          <label>Series naming template<input value={data.series_naming_template} disabled={!data.rename_enabled} onChange={e=>setData({...data,series_naming_template:e.target.value})}/></label>
          <small className="settings-help">Common tokens: {'{Title} {Year} {Resolution} {Source} {Codec} {HDR} {Audio} {Language}'}. Series also supports {'{Season} {Season:00} {Episode} {Episode:00} {EpisodeTitle}'}. Season packs are imported file-by-file when episode numbers can be detected.</small>
        </section>

        <section className="settings-section card">
          <div className="settings-title"><div><h3>Paths</h3><span>Media, downloads, reseed workspace and metadata.</span></div></div>
          <div className="settings-path-grid">
            <label>Movies<input value={data.movies_path} onChange={e=>setData({...data,movies_path:e.target.value})}/><FolderPicker value={data.movies_path} onChange={movies_path=>setData({...data,movies_path})}/></label>
            <label>Series<input value={data.series_path} onChange={e=>setData({...data,series_path:e.target.value})}/><FolderPicker value={data.series_path} onChange={series_path=>setData({...data,series_path})}/></label>
            <label>Downloads<input value={data.downloads_path} onChange={e=>setData({...data,downloads_path:e.target.value})}/><FolderPicker value={data.downloads_path} onChange={downloads_path=>setData({...data,downloads_path})}/></label>
            <label>Reseed workspace<input value={data.reseed_path} onChange={e=>setData({...data,reseed_path:e.target.value})} placeholder="Blank = downloads/reseed"/><FolderPicker value={data.reseed_path} onChange={reseed_path=>setData({...data,reseed_path})}/></label>
            <label>Torrent metadata<input value={data.torrent_metadata_path} onChange={e=>setData({...data,torrent_metadata_path:e.target.value})}/><FolderPicker value={data.torrent_metadata_path} onChange={torrent_metadata_path=>setData({...data,torrent_metadata_path})}/></label>
            <label>Custom indexers<input value={data.custom_indexers_path} onChange={e=>setData({...data,custom_indexers_path:e.target.value})}/><FolderPicker value={data.custom_indexers_path} onChange={custom_indexers_path=>setData({...data,custom_indexers_path})}/></label>
            <label>Upstream indexers<input value={data.upstream_indexers_path} onChange={e=>setData({...data,upstream_indexers_path:e.target.value})}/><FolderPicker value={data.upstream_indexers_path} onChange={upstream_indexers_path=>setData({...data,upstream_indexers_path})}/></label>
          </div>
        </section>

        <section className="settings-section card">
          <div className="settings-title"><div><h3>Library Maintenance</h3><span>Discover existing media files and rebuild real availability.</span></div></div>
          <div className="settings-actions settings-scan-actions">
            <span>{scanMessage}</span>
            <button type="button" className="ghost-button" onClick={()=>void scan('movie')}>Rescan Movies</button>
            <button type="button" className="ghost-button" onClick={()=>void scan('series')}>Rescan Series</button>
            <button type="button" className="primary-button" onClick={()=>void scan('all')}>Rescan All</button>
          </div>
          <small className="settings-help">Rescan does not move or rename files. It indexes files already present in the configured library paths, refreshes Movies availability and links SxxExx files to Series episodes.</small>
        </section>

        <section className="settings-section card">
          <div className="settings-title"><div><h3>Diagnostics</h3><span>Read-only status for support and troubleshooting. Secrets are never included.</span></div></div>
          <div className="settings-actions"><span>{diagnosticsMessage}</span><button type="button" className="ghost-button" onClick={()=>void loadDiagnostics()}>Refresh diagnostics</button></div>
          {diagnostics&&<div className="settings-help">
            <p>Oberiz {diagnostics.version} · {diagnostics.operating_system} · Database: {diagnostics.database}</p>
            <p>TMDB: {diagnostics.tmdb_configured?'configured':'not configured'} · qBittorrent: {diagnostics.qbittorrent_configured?'configured':'not configured'} · Indexers: {diagnostics.enabled_indexers}/{diagnostics.total_indexers} enabled · Automation: {diagnostics.automation_enabled?'enabled':'disabled'}</p>
            {diagnostics.recent_errors.length>0&&<p>Recent errors: {diagnostics.recent_errors.map(error=>`${error.source}: ${error.message}`).join(' · ')}</p>}
            {diagnostics.recent_errors.length===0&&<p>No recent errors recorded.</p>}
          </div>}
        </section>

        <section className="settings-section card">
          <div className="settings-title"><div><h3>Backup & Restore</h3><span>Consistent SQLite snapshots, including data currently in WAL.</span></div></div>
          <div className="form-grid automation-grid">
            <label className="toggle-row"><input type="checkbox" checked={data.backup_enabled} onChange={e=>setData({...data,backup_enabled:e.target.checked})}/> Schedule automatic backups</label>
            <label>Interval (hours)<input type="number" min="1" max="720" value={data.backup_interval_hours} disabled={!data.backup_enabled} onChange={e=>setData({...data,backup_interval_hours:Number(e.target.value)})}/></label>
            <label>Keep latest copies<input type="number" min="1" max="100" value={data.backup_retention_count} disabled={!data.backup_enabled} onChange={e=>setData({...data,backup_retention_count:Number(e.target.value)})}/></label>
          </div>
          <div className="settings-actions"><span>{backupMessage}</span><label className="ghost-button file-button">Upload backup<input type="file" accept=".sqlite,.sqlite3,.db,application/vnd.sqlite3" onChange={e=>void upload(e)}/></label><button type="button" className="primary-button" onClick={()=>void makeBackup()}>Create Backup</button></div>
          <small className="settings-help">Backups are stored in <b>{backups?.directory||'backups'}</b>. Restoring replaces all current database data; media files and qBittorrent are not changed.</small>
          {backups&&<div className="backup-list">
            {backups.backups.map(backup=><div className="backup-row" key={backup.filename}><div><strong>{backup.filename}</strong><span>{new Date(backup.created_at).toLocaleString()} · {formatBytes(backup.size_bytes)}</span></div><div className="backup-actions"><a className="ghost-button" href={backupDownloadUrl(backup.filename)}>Download</a><button type="button" className="danger-button" onClick={()=>void removeBackup(backup.filename)}>Delete</button><button type="button" className="danger-button" onClick={()=>void restore(backup.filename)}>Restore</button></div></div>)}
            {backups.backups.length===0&&<p className="settings-help">No backups created yet.</p>}
          </div>}
        </section>

        <section className="settings-section card api-key-section">
          <div className="settings-title"><div><h3>Public API v1</h3><span>Direct integration for your multimedia app or other request clients.</span></div><span className="connection">{data.api_enabled?'● Enabled':'○ Disabled'}</span></div>
          <label className="toggle-row"><input type="checkbox" checked={data.api_enabled} onChange={e=>setData({...data,api_enabled:e.target.checked})}/> Enable public request API</label>

          <label>API key
            <div className="api-key-field">
              <input
                type={apiKeyVisible?'text':'password'}
                value={apiKey}
                onChange={e=>{setApiKey(e.target.value);setApiCopied(false)}}
                placeholder={data.api_key_set?'Current API key':'Generate or enter a key'}
                autoComplete="off"
              />
              <button type="button" className="ghost-button" disabled={!apiKey} onClick={()=>setApiKeyVisible(v=>!v)}>{apiKeyVisible?'Hide':'Show'}</button>
              <button type="button" className="ghost-button" disabled={!apiKey} onClick={()=>void copyApiKey()}>{apiCopied?'Copied':'Copy'}</button>
            </div>
          </label>

          {apiKey&&<div className="api-key-warning"><b>API key configured.</b><span>The key remains available here after reload. Use Show to reveal it and Copy to copy it.</span></div>}
          <div className="settings-actions"><span>{apiMessage}</span><button type="button" className="ghost-button" onClick={generateApiKey}>{data.api_key_set?'Regenerate key':'Generate key'}</button></div>
          <small className="settings-help">Clients send <b>X-Api-Key</b>. Endpoints: GET /api/v1/status, POST/GET /api/v1/requests, GET /api/v1/requests/:id.</small>
          <label className="toggle-row"><input type="checkbox" checked={data.overseerr_compat_enabled} onChange={e=>setData({...data,overseerr_compat_enabled:e.target.checked})}/> Enable Overseerr compatibility</label>
          {data.overseerr_compat_enabled&&<small className="settings-help">Configure the same Oberiz URL twice in Overseerr: <b>http://host:2032/radarr</b> for Movies and <b>http://host:2032/sonarr</b> for Series. Use this API key in both.</small>}
        </section>

        <section className="settings-section card">
          <div className="settings-title"><div><h3>Seed Policies</h3><span>Per-indexer cleanup rules. No policy means no automatic deletion.</span></div></div>
          {policies.length>0&&<div className="seed-policy-list">
            {policies.map(p=><button type="button" className="seed-policy-row" key={p.indexer_id} onClick={()=>setPolicy(p)}>
              <strong>{p.indexer_id}</strong>
              <span>{(p.min_seed_time_minutes/1440).toFixed(1)} days · ratio {p.min_ratio.toFixed(2)} · {p.requirement_mode}</span>
              <em>{p.cleanup_mode}</em>
            </button>)}
          </div>}
          <div className="seed-policy-editor">
            <label>Indexer ID<input value={policy.indexer_id} onChange={e=>setPolicy({...policy,indexer_id:e.target.value})} placeholder="e.g. my-private-tracker"/></label>
            <label>Minimum seed days<input type="number" min="0" step=".25" value={policy.min_seed_time_minutes/1440} onChange={e=>setPolicy({...policy,min_seed_time_minutes:Math.round(Number(e.target.value)*1440)})}/></label>
            <label>Minimum ratio<input type="number" min="0" step=".1" value={policy.min_ratio} onChange={e=>setPolicy({...policy,min_ratio:Number(e.target.value)})}/></label>
            <label>Requirement
              <select value={policy.requirement_mode} onChange={e=>setPolicy({...policy,requirement_mode:e.target.value as SeedPolicy['requirement_mode']})}>
                <option value="manual">Manual</option><option value="time">Time</option><option value="ratio">Ratio</option><option value="either">Either</option><option value="both">Both</option>
              </select>
            </label>
            <label>Cleanup
              <select value={policy.cleanup_mode} onChange={e=>setPolicy({...policy,cleanup_mode:e.target.value as SeedPolicy['cleanup_mode']})}>
                <option value="manual">Manual</option><option value="never">Never auto-delete</option>
                <option value="remove_torrent_and_original">Remove torrent + original download</option>
                <option value="remove_torrent_keep_files">Remove torrent, keep original download</option>
              </select>
            </label>
            <button type="button" className="ghost-button" onClick={()=>void savePolicy()}>Save Policy</button>
          </div>
          <div className="settings-actions"><span>{policyMessage}</span></div>
        </section>

        <div className="settings-actions"><span>{message}</span><button className="primary-button" type="submit">Save Settings</button></div>
      </div>
    </form>
  </>
}

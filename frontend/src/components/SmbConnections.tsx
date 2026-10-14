import { useEffect, useState } from 'react'
import { t } from '../lib/i18n'
type Connection = {server:string;username:string;shares:string[];password_set:boolean}
export function SmbConnections(){
  const [connections,setConnections]=useState<Connection[]>([])
  const [server,setServer]=useState(''),[username,setUsername]=useState(''),[password,setPassword]=useState(''),[share,setShare]=useState('')
  const [busy,setBusy]=useState(false),[message,setMessage]=useState('')
  async function load(){const response=await fetch('/api/storage/smb');if(!response.ok)throw new Error(await response.text());setConnections(await response.json())}
  useEffect(()=>{void load().catch(error=>setMessage(String(error)))},[])
  async function connect(){
    setBusy(true);setMessage('')
    try{const response=await fetch('/api/storage/smb',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({server,username,password:password||undefined,share:share||undefined})});if(!response.ok)throw new Error(await response.text());setPassword('');await load();setMessage(t('SMB connection saved'))}
    catch(error){setMessage(error instanceof Error?error.message:String(error))}finally{setBusy(false)}
  }
  async function forget(server:string){
    if(!confirm(t('Forget this saved SMB connection? Existing Windows sessions will not be disconnected.')))return
    setBusy(true);setMessage('')
    try{const response=await fetch('/api/storage/smb',{method:'DELETE',headers:{'Content-Type':'application/json'},body:JSON.stringify({server})});if(!response.ok)throw new Error(await response.text());await load()}
    catch(error){setMessage(error instanceof Error?error.message:String(error))}finally{setBusy(false)}
  }
  return <div className="smb-connections">
    <h4>{t('SMB / CIFS servers')}</h4>
    <p className="settings-help">{t('Use HTTPS when entering credentials remotely. Saved passwords are protected by Windows; shared folders appear in the folder picker.')}</p>
    <div className="settings-path-grid">
      <label>{t('Server')}<input value={server} onChange={event=>setServer(event.target.value)} placeholder="192.168.12.200" autoComplete="off"/></label>
      <label>{t('Username')}<input value={username} onChange={event=>setUsername(event.target.value)} autoComplete="off"/></label>
      <label>{t('Password')}<input type="password" value={password} onChange={event=>setPassword(event.target.value)} autoComplete="new-password" placeholder={t('Leave empty to keep the saved password')}/></label>
      <label>{t('Share name (optional)')}<input value={share} onChange={event=>setShare(event.target.value)} placeholder="wd19tb" autoComplete="off"/></label>
    </div>
    <button type="button" className="ghost-button" disabled={busy||!server.trim()} onClick={()=>void connect()}>{busy?t('Connecting…'):t('Connect and save')}</button>
    {message&&<p role="status">{message}</p>}
    {connections.map(connection=><div key={connection.server}><button type="button" className="ghost-button" onClick={()=>{setServer(connection.server);setUsername(connection.username);setPassword('');setShare('')}}>{connection.server} · {connection.username}</button> <button type="button" className="danger-button" disabled={busy} onClick={()=>void forget(connection.server)}>{t('Forget saved connection')}</button><p className="settings-help">{connection.shares.join(' · ')||t('No shares discovered')}</p></div>)}
  </div>
}

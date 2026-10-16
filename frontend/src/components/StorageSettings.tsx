import { ReactNode } from 'react'
import { Settings as SettingsType } from '../lib/api'
import { t, useTranslation } from '../lib/i18n'

export function StorageSettings({data,setData,existingLibrary}:{
  data:SettingsType; setData:(data:SettingsType)=>void; existingLibrary:ReactNode
}){
  useTranslation()
  const stale=!data.last_external_report||!Number.isFinite(Date.parse(data.last_external_report))||Date.now()-Date.parse(data.last_external_report)>24*60*60*1000
  return <section className="settings-section card settings-full storage-settings">
    <div className="settings-title"><div><h3>{t('Library inventory')}</h3><span>{t('Choose one source. Download destinations are managed in qBittorrent categories.')}</span></div></div>
    <div className="storage-choices">
      <label className="storage-choice"><input type="radio" name="library-source" checked={data.library_source==='cinetta'} onChange={()=>setData({...data,library_source:'cinetta'})}/><span><b>Cinetta</b><small>{t('Receive the library through the public API. No NAS access required.')}</small></span></label>
      <label className="storage-choice"><input type="radio" name="library-source" checked={data.library_source==='local'} onChange={()=>setData({...data,library_source:'local'})}/><span><b>{t('Local folders')}</b><small>{t('Read-only scanning. Oberiz never imports, copies or moves media files.')}</small></span></label>
    </div>
    {data.library_source==='cinetta'
      ?<div className="storage-step"><p className="settings-help">{t('Enable Public API v1, generate a key and save Settings. In Cinetta, configure the Oberiz URL and key and enable inventory reporting.')}</p><p className="settings-help"><code>PUT /api/v1/library</code> · {t('Cinetta sends the inventory; selecting this mode does not initiate a sync.')}</p><p role="status">{t('Last inventory received:')} {data.last_external_report?new Date(data.last_external_report).toLocaleString():t('Not received yet')}</p><p className="settings-help">{t('If Cinetta stops reporting, the last inventory is preserved. Local scanning is never enabled automatically.')}</p></div>
      :<div className="storage-step">{existingLibrary}</div>}
    <p className="settings-help">{t('Optional renaming is configured separately and performed only through qBittorrent. Recovery metadata is included in Oberiz database backups.')}</p>
    {data.library_source==='cinetta'&&<label className="toggle-row"><input type="checkbox" checked={data.pause_on_stale_inventory??false} onChange={event=>setData({...data,pause_on_stale_inventory:event.target.checked})}/><span>{t('Pause new acquisitions if Cinetta inventory is missing or older than 24 hours')}</span></label>}
    {data.library_source==='cinetta'&&data.pause_on_stale_inventory&&stale&&<p role="alert" className="settings-help">{t('No recent Cinetta inventory: automatic acquisition is held until a report is received (maximum age: 24 hours).')}</p>}
  </section>
}

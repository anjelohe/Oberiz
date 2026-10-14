import { ReactNode, useState } from 'react'
import { getPathMappingSuggestions, MappingSuggestion, Settings as SettingsType } from '../lib/api'
import { FolderPicker } from './FolderPicker'
import { SmbConnections } from './SmbConnections'
import { t, tf, useTranslation } from '../lib/i18n'

type Mapping = { remote:string; local:string }
const pathKey = (path:string) => path.trim().replace(/\\/g,'/').replace(/\/+$/,'').toLowerCase()
const isNetworkPath = (path:string) => /^(\\\\|\/\/)/.test(path.trim())
const suggestionKey = (s:MappingSuggestion) => `${s.remote}\u0000${s.local}`

/** Adds mappings whose remote side is not mapped yet; existing rows always win. */
export function mergeMappings(current:Mapping[], added:Mapping[]):Mapping[]{
  const merged=current.filter(m=>m.remote.trim()||m.local.trim())
  for(const mapping of added)if(!merged.some(m=>pathKey(m.remote)===pathKey(mapping.remote)))merged.push(mapping)
  return merged
}

/**
 * Everything about where files live, as one guided block: whether Oberiz
 * manages files at all, where qBittorrent runs (and the path mappings that
 * implies), the folders it uses, NAS access, and the optional scan of an
 * existing library. Only the steps the current answers need are shown.
 */
export function StorageSettings({data,setData,qbRemote,setQbRemote,existingLibrary,hasExistingLibrary}:{
  data:SettingsType; setData:(data:SettingsType)=>void; qbRemote:boolean; setQbRemote:(remote:boolean)=>void;
  existingLibrary:ReactNode; hasExistingLibrary:boolean
}){
  useTranslation()
  const managed=data.import_method!=='external'
  const [managedMethod,setManagedMethod]=useState(managed?data.import_method:'auto')
  const [suggestions,setSuggestions]=useState<MappingSuggestion[]|null>(null)
  const [accepted,setAccepted]=useState<string[]>([])
  const [detecting,setDetecting]=useState(false),[detectMessage,setDetectMessage]=useState('')
  const [showSmb,setShowSmb]=useState(false)
  const mappings=data.qbittorrent_path_mappings
  const setMappings=(next:Mapping[])=>setData({...data,qbittorrent_path_mappings:next})
  const usedPaths=[
    ...(managed?[data.downloads_path,data.movies_path,data.series_path,...(qbRemote?mappings.map(m=>m.local):[])]:[]),
    ...data.movie_library_paths,...data.series_library_paths,
  ]
  const needsSmb=usedPaths.some(isNetworkPath)
  let step=0
  const next=()=>++step

  async function detect(){
    setDetecting(true);setDetectMessage('');setSuggestions(null)
    try{
      const found=await getPathMappingSuggestions()
      setSuggestions(found)
      setAccepted(found.filter(s=>s.verified).map(suggestionKey))
      if(!found.length)setDetectMessage(t('No mapping needed: Oberiz already opens every folder qBittorrent uses, or no matching shared folder was found.'))
    }catch(e){setDetectMessage(e instanceof Error?e.message:String(e))}
    finally{setDetecting(false)}
  }
  function addAccepted(){
    const chosen=(suggestions??[]).filter(s=>accepted.includes(suggestionKey(s)))
    setMappings(mergeMappings(mappings,chosen.map(s=>({remote:s.remote,local:s.local}))))
    setSuggestions(null)
    setDetectMessage(tf('{0} mappings added. Save Settings to apply them.',chosen.length))
  }

  return <section className="settings-section card settings-full storage-settings">
    <div className="settings-title"><div><h3>{t('Files and storage')}</h3><span>{t('Where qBittorrent downloads, where your movies and series end up, and how Oberiz reaches them.')}</span></div></div>

    <div className="storage-step">
      <h4>{next()}. {t('Should Oberiz move and rename downloads?')}</h4>
      <div className="storage-choices">
        <label className="storage-choice"><input type="radio" name="storage-manage" checked={managed} onChange={()=>setData({...data,import_method:managedMethod&&managedMethod!=='external'?managedMethod:'auto'})}/><span><b>{t('Yes')}</b><small>{t('Oberiz imports each completed download into your library folder and renames it.')}</small></span></label>
        <label className="storage-choice"><input type="radio" name="storage-manage" checked={!managed} onChange={()=>{if(managed)setManagedMethod(data.import_method);setData({...data,import_method:'external'})}}/><span><b>{t('No')}</b><small>{t('qBittorrent downloads straight into your library and your media server (Cinetta) tells Oberiz what you have.')}</small></span></label>
      </div>
      {!managed&&<p className="settings-help">{t('Oberiz will never open, move, rename or delete files. Point each qBittorrent category directly at a library folder. Folders, NAS access and path mappings are not needed for downloads.')}</p>}
    </div>

    {managed&&<div className="storage-step">
      <h4>{next()}. {t('Where does qBittorrent run?')}</h4>
      <div className="storage-choices">
        <label className="storage-choice"><input type="radio" name="storage-qb" checked={!qbRemote} onChange={()=>setQbRemote(false)}/><span><b>{t('On this computer')}</b><small>{t('Oberiz and qBittorrent see the same paths. Nothing to configure.')}</small></span></label>
        <label className="storage-choice"><input type="radio" name="storage-qb" checked={qbRemote} onChange={()=>setQbRemote(true)}/><span><b>{t('On another computer or the NAS')}</b><small>{t('qBittorrent reports its own paths, which Oberiz must translate into folders it can open.')}</small></span></label>
      </div>
      {qbRemote&&<div className="storage-mappings">
        <p className="settings-help">{t('Example: qBittorrent sees /volume1/WD19TB and Oberiz sees')} <code>\\192.168.12.200\WD19TB</code>.</p>
        {mappings.map((mapping,i)=><div className="library-mapping-row" key={i}>
          <label>{t('Remote path (as qBittorrent sees it)')}<input value={mapping.remote} onChange={e=>setMappings(mappings.map((m,j)=>j===i?{...m,remote:e.target.value}:m))} placeholder="/volume1/WD19TB"/></label>
          <label>{t('Local path (as Oberiz sees it)')}<input value={mapping.local} onChange={e=>setMappings(mappings.map((m,j)=>j===i?{...m,local:e.target.value}:m))} placeholder="\\192.168.12.200\WD19TB"/><FolderPicker value={mapping.local} onChange={local=>setMappings(mappings.map((m,j)=>j===i?{...m,local}:m))}/></label>
          <button type="button" className="danger-button" onClick={()=>setMappings(mappings.filter((_,j)=>j!==i))}>{t('Remove')}</button>
        </div>)}
        <div className="settings-actions storage-mapping-actions">
          <span role="status">{detectMessage}</span>
          <button type="button" className="ghost-button" disabled={detecting} onClick={()=>void detect()}>{detecting?t('Detecting…'):t('Detect automatically')}</button>
          <button type="button" className="ghost-button" onClick={()=>setMappings([...mappings,{remote:'',local:''}])}>{t('Add mapping')}</button>
        </div>
        {suggestions&&suggestions.length>0&&<div className="storage-suggestions">
          <strong>{t('Suggested mappings')}</strong>
          {suggestions.map(s=><label className="storage-suggestion" key={suggestionKey(s)}>
            <input type="checkbox" checked={accepted.includes(suggestionKey(s))} onChange={e=>setAccepted(keys=>e.target.checked?[...keys,suggestionKey(s)]:keys.filter(k=>k!==suggestionKey(s)))}/>
            <span><code>{s.remote}</code> → <code>{s.local}</code>
              <small className={s.verified?'verified':'unverified'}>{s.verified?tf('Checked: {0} opens correctly.',s.example_local):tf('Not verified: {0} could not be opened. If the NAS needs a password, connect it in the NAS access step first.',s.example_local)}</small>
            </span>
          </label>)}
          <div className="settings-actions"><button type="button" className="ghost-button" disabled={!accepted.length} onClick={addAccepted}>{t('Add selected mappings')}</button></div>
        </div>}
      </div>}
    </div>}

    {managed&&<div className="storage-step">
      <h4>{next()}. {t('Folders')}</h4>
      <div className="settings-path-grid storage-folders">
        <label>{t('Downloads')}<input value={data.downloads_path} onChange={e=>setData({...data,downloads_path:e.target.value})}/><FolderPicker value={data.downloads_path} onChange={downloads_path=>setData({...data,downloads_path})}/><small className="settings-help">{t('Where qBittorrent leaves completed downloads, as Oberiz sees it.')}</small></label>
        <label>{t('Movie import destination')}<input value={data.movies_path} onChange={e=>setData({...data,movies_path:e.target.value})}/><FolderPicker value={data.movies_path} onChange={movies_path=>setData({...data,movies_path})}/><small className="settings-help">{t('Where movies end up, already renamed.')}</small></label>
        <label>{t('Series import destination')}<input value={data.series_path} onChange={e=>setData({...data,series_path:e.target.value})}/><FolderPicker value={data.series_path} onChange={series_path=>setData({...data,series_path})}/><small className="settings-help">{t('Where series end up, already renamed.')}</small></label>
      </div>
    </div>}

    {(needsSmb||showSmb)
      ?<div className="storage-step">
        <h4>{next()}. {t('NAS access (SMB)')}</h4>
        <p className="settings-help">{t('The NAS username and password, so the Oberiz service can open folders that start with \\\\. Not needed for folders on this computer.')}</p>
        <SmbConnections/>
      </div>
      :<div className="settings-actions storage-smb-toggle"><button type="button" className="ghost-button" onClick={()=>setShowSmb(true)}>{t('Configure NAS access (SMB)')}</button></div>}

    {managed&&<details className="storage-details">
      <summary>{t('Advanced options')}</summary>
      <div className="settings-path-grid">
        <label>{t('Reseed workspace')}<input value={data.reseed_path} onChange={e=>setData({...data,reseed_path:e.target.value})} placeholder={t('Blank = downloads/reseed')}/><FolderPicker value={data.reseed_path} onChange={reseed_path=>setData({...data,reseed_path})}/></label>
        <label>{t('Torrent metadata')}<input value={data.torrent_metadata_path} onChange={e=>setData({...data,torrent_metadata_path:e.target.value})}/><FolderPicker value={data.torrent_metadata_path} onChange={torrent_metadata_path=>setData({...data,torrent_metadata_path})}/></label>
      </div>
    </details>}

    <details className="storage-details" open={hasExistingLibrary}>
      <summary>{t('Existing library (optional)')}<small>{t('Folders Oberiz scans to detect what you already had. Not needed if your media server (Cinetta) reports your library.')}</small></summary>
      {existingLibrary}
    </details>
  </section>
}

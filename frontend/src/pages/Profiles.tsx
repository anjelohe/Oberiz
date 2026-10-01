import { useEffect, useMemo, useState } from 'react'
import {
  createLanguageProfile, createQualityProfile, deleteLanguageProfile, deleteQualityProfile,
  getLanguageProfiles, getQualityProfiles, getQBittorrentCategories, LanguageProfile, QualityProfile, QualityRules,
  QBittorrentCategory, saveLanguageProfile, saveQualityProfile, setDefaultQualityProfile,
} from '../lib/api'
import { Icon } from '../components/Icon'
import { useModalA11y } from '../lib/useModalA11y'
import './Profiles.css'

type Tab='movie'|'series'|'language'
type RuleGroup='resolutions'|'sources'|'codecs'|'hdr'|'audio'

const OPTIONS:Record<RuleGroup,string[]>={
  resolutions:['2160P','1080P','720P','480P'],
  sources:['REMUX','BluRay','WEB-DL','WEBRip','HDTV'],
  codecs:['x265','HEVC','AV1','x264','H.264'],
  hdr:['Dolby Vision','HDR10+','HDR'],
  audio:['Atmos','TrueHD','DTS-HD','DTS','DD+','AAC','FLAC'],
}
const LANGUAGES=['Spanish','Castellano','Dual','Latino','Multi','English']

function blankRules():QualityRules{return {
  resolutions:{'1080P':250},sources:{'WEB-DL':120,'BluRay':130},codecs:{},hdr:{},audio:{},
  reject_terms:['CAM','TELESYNC'],prefer_terms:{},allow_unknown_resolution:false,allow_unknown_source:true,
  series_prefer_pack:false,prefer_indexer_priority:false,series_accept_complete:true,
}}

function blankQuality(media_type:'movie'|'series',languageId:number|null):Omit<QualityProfile,'id'|'language_profile_name'|'is_default'|'created_at'|'updated_at'>{
  return {name:media_type==='movie'?'New Movie Profile':'New Series Profile',media_type,enabled:true,upgrade_allowed:true,
    cutoff_score:500,min_seeders:1,min_size_mb:null,max_size_mb:null,max_season_pack_size_mb:null,language_profile_id:languageId,
    qbittorrent_category:'',qbittorrent_tags_template:'[tracker]',request_quality:'standard',rules:blankRules()}
}

function qualityDraft(profile:QualityProfile):Omit<QualityProfile,'id'|'language_profile_name'|'is_default'|'created_at'|'updated_at'>{
  const {id,language_profile_name,is_default,created_at,updated_at,...draft}=profile
  return {...draft,rules:JSON.parse(JSON.stringify(draft.rules))}
}

function QualityEditor({profile,cloneFrom,initialMedia,languages,onClose,onSaved}:{profile:QualityProfile|null;cloneFrom?:QualityProfile|null;initialMedia:'movie'|'series';languages:LanguageProfile[];onClose:()=>void;onSaved:()=>Promise<void>}){
  const [draft,setDraft]=useState<Omit<QualityProfile,'id'|'language_profile_name'|'is_default'|'created_at'|'updated_at'>>(
    profile ? qualityDraft(profile) : cloneFrom ? {...qualityDraft(cloneFrom),name:`${cloneFrom.name} Copy`} : blankQuality(initialMedia,languages[0]?.id??null)
  )
  const [message,setMessage]=useState('')
  const [categories,setCategories]=useState<QBittorrentCategory[]>([])
  const [categoriesError,setCategoriesError]=useState('')
  // A plain string draft, not derived from draft.rules.prefer_terms on every
  // render: re-serializing the parsed map back to text on each keystroke
  // rewrote whatever the user had just typed (typing "P" parsed to {P:0},
  // which re-rendered as "P:0" before they could type anything else).
  const [preferTermsText,setPreferTermsText]=useState(()=>
    Object.entries(draft.rules.prefer_terms).map(([k,v])=>`${k}:${v}`).join(', ')
  )
  const dialogRef=useModalA11y<HTMLDivElement>(true,onClose)

  async function loadCategories(){
    setCategoriesError('')
    try{setCategories((await getQBittorrentCategories()).categories)}
    catch(e){setCategoriesError(e instanceof Error?e.message:String(e))}
  }
  useEffect(()=>{void loadCategories()},[])

  function setRule(group:RuleGroup,key:string,enabled:boolean,score?:number){
    setDraft(current=>{
      const next={...current,rules:{...current.rules,[group]:{...current.rules[group]}}}
      if(enabled) next.rules[group][key]=score ?? next.rules[group][key] ?? 0
      else delete next.rules[group][key]
      return next
    })
  }

  async function save(){
    setMessage('Saving…')
    try{
      if(profile) await saveQualityProfile(profile.id,draft)
      else await createQualityProfile(draft)
      await onSaved();onClose()
    }catch(e){setMessage(e instanceof Error?e.message:String(e))}
  }

  return <div className="profile-modal-backdrop" onMouseDown={onClose}>
    <div ref={dialogRef} className="profile-modal" role="dialog" aria-modal="true" aria-labelledby="quality-editor-title" tabIndex={-1} onMouseDown={e=>e.stopPropagation()}>
      <div className="profile-modal-head"><div><span>QUALITY PROFILE</span><h2 id="quality-editor-title">{profile?'Edit profile':cloneFrom?'Clone profile':'New profile'}</h2></div><button onClick={onClose} aria-label="Close">×</button></div>
      <div className="profile-form-grid">
        <label>Name<input value={draft.name} onChange={e=>setDraft({...draft,name:e.target.value})}/></label>
        <label>Media type<select value={draft.media_type} onChange={e=>setDraft({...draft,media_type:e.target.value as 'movie'|'series'})}><option value="movie">Movies</option><option value="series">Series</option></select></label>
        <label>Language profile<select value={draft.language_profile_id??''} onChange={e=>setDraft({...draft,language_profile_id:e.target.value?Number(e.target.value):null})}><option value="">None</option>{languages.map(x=><option key={x.id} value={x.id}>{x.name}</option>)}</select></label>
        <label>Minimum seeds<input type="number" min="0" value={draft.min_seeders} onChange={e=>setDraft({...draft,min_seeders:Number(e.target.value)})}/></label>
        <label>Min size MB<input type="number" value={draft.min_size_mb??''} onChange={e=>setDraft({...draft,min_size_mb:e.target.value===''?null:Number(e.target.value)})}/></label>
        <label>Max episode size MB<input type="number" value={draft.max_size_mb??''} onChange={e=>setDraft({...draft,max_size_mb:e.target.value===''?null:Number(e.target.value)})}/></label>
        {draft.media_type==='series'&&<label>Max season/pack size MB<input type="number" value={draft.max_season_pack_size_mb??''} onChange={e=>setDraft({...draft,max_season_pack_size_mb:e.target.value===''?null:Number(e.target.value)})}/></label>}
        <label>Upgrade cutoff score<input type="number" value={draft.cutoff_score} onChange={e=>setDraft({...draft,cutoff_score:Number(e.target.value)})}/></label>
      </div>
      <section className="profile-routing">
        <div className="profile-routing-head"><div><span>DOWNLOAD ROUTING</span><h3>qBittorrent</h3></div><small>Category controls destination. Tags can use variables.</small></div>
        <div className="profile-form-grid wide">
          <label>Default qBittorrent Category
            <div className="profile-category-select-row">
              <select value={draft.qbittorrent_category} onChange={e=>setDraft({...draft,qbittorrent_category:e.target.value})}>
                <option value="">Uncategorized / Sin categorizar</option>
                {draft.qbittorrent_category&&!categories.some(c=>c.name===draft.qbittorrent_category)&&<option value={draft.qbittorrent_category}>{categoriesError?'Current':'⚠ Missing'}: {draft.qbittorrent_category}</option>}
                {categories.map(c=><option key={c.name} value={c.name}>{c.name}{c.save_path?` — ${c.save_path}`:''}</option>)}
              </select>
              <button type="button" className="ghost-button" onClick={()=>void loadCategories()}>Refresh</button>
            </div>
            <small>{categoriesError?`qBittorrent: ${categoriesError}`:'Loaded directly from qBittorrent. Empty means truly uncategorized; save paths remain managed by qBittorrent.'}</small>
            {draft.qbittorrent_category&&categories.find(c=>c.name===draft.qbittorrent_category)?.save_path&&<strong className="profile-category-path">{categories.find(c=>c.name===draft.qbittorrent_category)?.save_path}</strong>}
          </label>
          <label>qBittorrent Tags
            <input value={draft.qbittorrent_tags_template} onChange={e=>setDraft({...draft,qbittorrent_tags_template:e.target.value})} placeholder="[tracker], [resolution], [profile]"/>
            <small>Comma separated. Variables are resolved at Grab time.</small>
          </label>
        </div>
        <div className="profile-token-list">
          <button type="button" onClick={()=>setDraft({...draft,qbittorrent_tags_template:`${draft.qbittorrent_tags_template}${draft.qbittorrent_tags_template.trim()?', ':''}[tracker]`})}>[tracker]</button>
          <button type="button" onClick={()=>setDraft({...draft,qbittorrent_tags_template:`${draft.qbittorrent_tags_template}${draft.qbittorrent_tags_template.trim()?', ':''}[profile]`})}>[profile]</button>
          <button type="button" onClick={()=>setDraft({...draft,qbittorrent_tags_template:`${draft.qbittorrent_tags_template}${draft.qbittorrent_tags_template.trim()?', ':''}[media_type]`})}>[media_type]</button>
          <button type="button" onClick={()=>setDraft({...draft,qbittorrent_tags_template:`${draft.qbittorrent_tags_template}${draft.qbittorrent_tags_template.trim()?', ':''}[resolution]`})}>[resolution]</button>
          <button type="button" onClick={()=>setDraft({...draft,qbittorrent_tags_template:`${draft.qbittorrent_tags_template}${draft.qbittorrent_tags_template.trim()?', ':''}[source]`})}>[source]</button>
          <button type="button" onClick={()=>setDraft({...draft,qbittorrent_tags_template:`${draft.qbittorrent_tags_template}${draft.qbittorrent_tags_template.trim()?', ':''}[codec]`})}>[codec]</button>
          <button type="button" onClick={()=>setDraft({...draft,qbittorrent_tags_template:`${draft.qbittorrent_tags_template}${draft.qbittorrent_tags_template.trim()?', ':''}[language]`})}>[language]</button>
          <button type="button" onClick={()=>setDraft({...draft,qbittorrent_tags_template:`${draft.qbittorrent_tags_template}${draft.qbittorrent_tags_template.trim()?', ':''}[year]`})}>[year]</button>
        </div>
        <p className="profile-routing-note">Example: category <b>seriesWD19_4k</b> + tags <b>[tracker], [resolution]</b> → qBittorrent gets the destination category and tags such as <b>HDO, 2160P</b>.</p>
      </section>
      <div className="profile-toggles">
        <label><input type="checkbox" checked={draft.enabled} onChange={e=>setDraft({...draft,enabled:e.target.checked})}/> Enabled</label>
        <label><input type="checkbox" checked={draft.upgrade_allowed} onChange={e=>setDraft({...draft,upgrade_allowed:e.target.checked})}/> Allow upgrades</label>
        <label><input type="checkbox" checked={draft.request_quality==='4k'} onChange={e=>setDraft({...draft,request_quality:e.target.checked?'4k':'standard'})}/> Perfil 4K para peticiones API</label>
        <label><input type="checkbox" checked={draft.rules.allow_unknown_resolution} onChange={e=>setDraft({...draft,rules:{...draft.rules,allow_unknown_resolution:e.target.checked}})}/> Allow unknown resolution</label>
        <label><input type="checkbox" checked={draft.rules.allow_unknown_source} onChange={e=>setDraft({...draft,rules:{...draft.rules,allow_unknown_source:e.target.checked}})}/> Allow unknown source</label>
        <label title="Cuando esté activo, una release aceptada de un indexador con menor prioridad se ordena antes que otra de un indexador menos preferido."><input type="checkbox" checked={draft.rules.prefer_indexer_priority} onChange={e=>setDraft({...draft,rules:{...draft.rules,prefer_indexer_priority:e.target.checked}})}/> Prioritize indexer</label>
        {draft.media_type==='series'&&<>
          <label><input type="checkbox" checked={draft.rules.series_prefer_pack} onChange={e=>setDraft({...draft,rules:{...draft.rules,series_prefer_pack:e.target.checked}})}/> Prefer season/complete packs</label>
          <label><input type="checkbox" checked={draft.rules.series_accept_complete} onChange={e=>setDraft({...draft,rules:{...draft.rules,series_accept_complete:e.target.checked}})}/> Accept complete series</label>
        </>}
      </div>
      <div className="quality-groups">
        {(Object.keys(OPTIONS) as RuleGroup[]).map(group=><section key={group} className="quality-group"><h3>{group}</h3>
          {OPTIONS[group].map(option=>{
            const enabled=Object.prototype.hasOwnProperty.call(draft.rules[group],option)
            return <div className="quality-rule" key={option}><label><input type="checkbox" checked={enabled} onChange={e=>setRule(group,option,e.target.checked)}/><span>{option}</span></label><input type="number" disabled={!enabled} value={enabled?draft.rules[group][option]:0} onChange={e=>setRule(group,option,true,Number(e.target.value))}/></div>
          })}
        </section>)}
      </div>
      <div className="profile-form-grid wide">
        <label>Reject terms<input value={draft.rules.reject_terms.join(', ')} onChange={e=>setDraft({...draft,rules:{...draft.rules,reject_terms:e.target.value.split(',').map(x=>x.trim()).filter(Boolean)}})} placeholder="CAM, TELESYNC, SCREENER"/></label>
        <label>Preferred terms (TERM:score)<input value={preferTermsText} onChange={e=>{
          const text=e.target.value
          setPreferTermsText(text)
          const values:Record<string,number>={};text.split(',').forEach(part=>{const [key,val]=part.split(':');if(key?.trim())values[key.trim()]=Number(val)||0});setDraft({...draft,rules:{...draft.rules,prefer_terms:values}})
        }} placeholder="PROPER:10, REPACK:10"/></label>
      </div>
      <div className="profile-modal-actions"><span>{message}</span><button className="ghost-button" onClick={onClose}>Cancel</button><button className="primary-button" onClick={()=>void save()}>Save Profile</button></div>
    </div>
  </div>
}

function LanguageEditor({profile,onClose,onSaved}:{profile:LanguageProfile|null;onClose:()=>void;onSaved:()=>Promise<void>}){
  const [draft,setDraft]=useState<Omit<LanguageProfile,'id'|'created_at'|'updated_at'>>(profile?{...profile,scores:{...profile.scores},allowed_languages:[...profile.allowed_languages]}:{name:'New Language Profile',allowed_languages:[...LANGUAGES],scores:{Spanish:120,Dual:100},allow_unknown:true})
  const [message,setMessage]=useState('')
  const dialogRef=useModalA11y<HTMLDivElement>(true,onClose)
  async function save(){try{profile?await saveLanguageProfile(profile.id,draft):await createLanguageProfile(draft);await onSaved();onClose()}catch(e){setMessage(e instanceof Error?e.message:String(e))}}
  return <div className="profile-modal-backdrop" onMouseDown={onClose}><div ref={dialogRef} className="profile-modal language-modal" role="dialog" aria-modal="true" aria-labelledby="language-editor-title" tabIndex={-1} onMouseDown={e=>e.stopPropagation()}>
    <div className="profile-modal-head"><div><span>LANGUAGE PROFILE</span><h2 id="language-editor-title">{profile?'Edit language profile':'New language profile'}</h2></div><button onClick={onClose} aria-label="Close">×</button></div>
    <label className="full-label">Name<input value={draft.name} onChange={e=>setDraft({...draft,name:e.target.value})}/></label>
    <label className="profile-check"><input type="checkbox" checked={draft.allow_unknown} onChange={e=>setDraft({...draft,allow_unknown:e.target.checked})}/> Allow releases where language cannot be identified</label>
    <div className="language-rules">{LANGUAGES.map(lang=>{
      const allowed=draft.allowed_languages.includes(lang)
      return <div className="language-rule" key={lang}><label><input type="checkbox" checked={allowed} onChange={e=>setDraft({...draft,allowed_languages:e.target.checked?[...draft.allowed_languages,lang]:draft.allowed_languages.filter(x=>x!==lang)})}/><strong>{lang}</strong></label><span>Score</span><input type="number" disabled={!allowed} value={draft.scores[lang]??0} onChange={e=>setDraft({...draft,scores:{...draft.scores,[lang]:Number(e.target.value)}})}/></div>
    })}</div>
    <div className="profile-modal-actions"><span>{message}</span><button className="ghost-button" onClick={onClose}>Cancel</button><button className="primary-button" onClick={()=>void save()}>Save Language Profile</button></div>
  </div></div>
}

export function Profiles(){
  const [tab,setTab]=useState<Tab>('movie')
  const [profiles,setProfiles]=useState<QualityProfile[]>([])
  const [languages,setLanguages]=useState<LanguageProfile[]>([])
  const [editQuality,setEditQuality]=useState<QualityProfile|null|undefined>(undefined)
  const [cloneQuality,setCloneQuality]=useState<QualityProfile|null>(null)
  const [newMedia,setNewMedia]=useState<'movie'|'series'>('movie')
  const [editLanguage,setEditLanguage]=useState<LanguageProfile|null|undefined>(undefined)
  const [error,setError]=useState('')

  async function reload(){try{const [p,l]=await Promise.all([getQualityProfiles(),getLanguageProfiles()]);setProfiles(p);setLanguages(l);setError('')}catch(e){setError(e instanceof Error?e.message:String(e))}}
  useEffect(()=>{void reload()},[])
  const visible=useMemo(()=>profiles.filter(x=>x.media_type===tab),[profiles,tab])

  return <>
    <div className="page-heading"><div><h1><Icon name="profiles" size={32}/> Profiles</h1><p>Independent movie, series and language rules for matching, scoring and automatic downloads.</p></div></div>
    <div className="profile-tabs"><button className={tab==='movie'?'active':''} onClick={()=>setTab('movie')}>Movie Profiles</button><button className={tab==='series'?'active':''} onClick={()=>setTab('series')}>Series Profiles</button><button className={tab==='language'?'active':''} onClick={()=>setTab('language')}>Language Profiles</button></div>
    {error&&<div className="error-box">{error}</div>}
    {tab!=='language'?<>
      <div className="profile-toolbar"><div><strong>{visible.length} profiles</strong><span>Each movie or series can use a different profile.</span></div><button className="primary-button" onClick={()=>{setNewMedia(tab as 'movie'|'series');setCloneQuality(null);setEditQuality(null)}}><Icon name="plus" size={17}/> New Profile</button></div>
      <div className="profile-card-grid">{visible.map(p=><article className="profile-card card" key={p.id}>
        <div className="profile-card-head"><div><span>{p.media_type==='movie'?'MOVIE':'SERIES'}</span><h3>{p.name}</h3></div><div className="profile-head-status">{p.is_default&&<span className="profile-default-badge">Default</span>}{p.request_quality==='4k'&&<span className="profile-4k-badge">4K</span>}<span className={p.enabled?'profile-live':'profile-off'}>{p.enabled?'Enabled':'Disabled'}</span></div></div>
        <div className="profile-meta"><div><span>Language</span><strong>{p.language_profile_name||'None'}</strong></div><div><span>Min seeds</span><strong>{p.min_seeders}</strong></div><div><span>Category</span><strong>{p.qbittorrent_category||'Uncategorized'}</strong></div><div><span>Tags</span><strong>{p.qbittorrent_tags_template||'None'}</strong></div></div>
        <div className="profile-chips">{Object.entries(p.rules.resolutions).map(([k,v])=><b key={k}>{k} +{v}</b>)}{Object.keys(p.rules.sources).slice(0,4).map(k=><b key={k}>{k}</b>)}</div>
        <div className="profile-card-actions">{!p.is_default&&<button className="default-button" onClick={async()=>{try{await setDefaultQualityProfile(p.id);await reload()}catch(e){setError(e instanceof Error?e.message:String(e))}}}>Set Default</button>}<button className="ghost-button" onClick={()=>{setCloneQuality(p);setEditQuality(null)}}>Clone</button><button className="ghost-button" onClick={()=>{setCloneQuality(null);setEditQuality(p)}}>Edit</button><button className="danger-button" disabled={p.is_default} title={p.is_default?'Choose another default profile before deleting':''} onClick={async()=>{if(confirm(`Delete ${p.name}?`)){try{await deleteQualityProfile(p.id);await reload()}catch(e){setError(e instanceof Error?e.message:String(e))}}}}>Delete</button></div>
      </article>)}</div>
    </>:<>
      <div className="profile-toolbar"><div><strong>{languages.length} language profiles</strong><span>Allowed languages and language-specific scoring.</span></div><button className="primary-button" onClick={()=>setEditLanguage(null)}><Icon name="plus" size={17}/> New Language Profile</button></div>
      <div className="profile-card-grid">{languages.map(p=><article className="profile-card card" key={p.id}><div className="profile-card-head"><div><span>LANGUAGE</span><h3>{p.name}</h3></div></div><div className="profile-chips">{p.allowed_languages.map(x=><b key={x}>{x} {p.scores[x]?`+${p.scores[x]}`:''}</b>)}</div><p className="profile-note">Unknown language: {p.allow_unknown?'allowed':'rejected'}</p><div className="profile-card-actions"><button className="ghost-button" onClick={()=>setEditLanguage(p)}>Edit</button><button className="danger-button" onClick={async()=>{if(confirm(`Delete ${p.name}?`)){try{await deleteLanguageProfile(p.id);await reload()}catch(e){setError(e instanceof Error?e.message:String(e))}}}}>Delete</button></div></article>)}</div>
    </>}
    {editQuality!==undefined&&<QualityEditor profile={editQuality} cloneFrom={cloneQuality} initialMedia={newMedia} languages={languages} onClose={()=>{setEditQuality(undefined);setCloneQuality(null)}} onSaved={reload}/>}
    {editLanguage!==undefined&&<LanguageEditor profile={editLanguage} onClose={()=>setEditLanguage(undefined)} onSaved={reload}/>} 
  </>
}

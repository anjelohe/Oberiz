// Every response is inspected for the 401 the backend sends when no admin
// password has been created yet ("setup_required") or the session cookie is
// missing/expired ("authentication_required"); the app-level gate re-checks
// /api/auth/status on this event instead of every call site having to
// special-case it.
const nativeFetch = window.fetch.bind(window)
window.fetch = async (...args: Parameters<typeof fetch>) => {
  const response = await nativeFetch(...args)
  if (response.status === 401) {
    try {
      const body = await response.clone().json()
      if (body && (body.error === 'authentication_required' || body.error === 'setup_required')) {
        window.dispatchEvent(new CustomEvent('oberiz-auth-required'))
      }
    } catch { /* not a JSON body, ignore */ }
  }
  return response
}

export type AuthStatus = { enabled: boolean; authenticated: boolean }

export async function getAuthStatus(): Promise<AuthStatus> {
  const res = await fetch('/api/auth/status')
  if (!res.ok) throw new Error(await parseError(res))
  return res.json()
}
export async function login(password: string): Promise<void> {
  const res = await fetch('/api/auth/login', {
    method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ password }),
  })
  if (!res.ok) throw new Error(await parseError(res))
}
export async function logout(): Promise<void> {
  await fetch('/api/auth/logout', { method: 'POST' })
}
export async function setAdminPassword(currentPassword: string | null, newPassword: string | null): Promise<void> {
  const res = await fetch('/api/auth/password', {
    method: 'POST', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ current_password: currentPassword, new_password: newPassword }),
  })
  if (!res.ok) throw new Error(await parseError(res))
}

export type Movie = {
  id: number; tmdb_id: number; title: string; original_title?: string | null;
  year?: number | null; overview?: string | null; poster_path?: string | null;
  backdrop_path?: string | null; monitored: boolean; library_path?: string | null;
  quality_profile_id?: number | null; quality_profile_name?: string | null;
  file_count:number; available:boolean; current_resolution:string|null;
  current_quality_score:number|null; upgrade_wanted:boolean;
  created_at: string; updated_at: string;
}

export type Series = {
  id: number; tmdb_id: number; name: string; original_name?: string | null;
  year?: number | null; overview?: string | null; poster_path?: string | null;
  backdrop_path?: string | null; monitored: boolean; library_path?: string | null;
  quality_profile_id?: number | null; quality_profile_name?: string | null;
  monitor_mode:string; metadata_synced_at?:string|null;
  season_count:number; episode_count:number; monitored_episode_count:number;
  available_episode_count:number; missing_episode_count:number; future_episode_count:number;
  latest_season_number:number|null;
  next_missing_season:number|null; next_missing_episode:number|null; next_missing_name:string|null;
  next_upcoming_season:number|null; next_upcoming_episode:number|null; next_upcoming_name:string|null; next_upcoming_air_date:string|null;
  created_at: string; updated_at: string;
}


export type SeriesEpisode = {
  id:number; series_id:number; season_id:number; tmdb_episode_id:number|null;
  season_number:number; episode_number:number; name:string; overview:string|null;
  air_date:string|null; still_path:string|null; runtime:number|null;
  monitored:boolean; has_file:boolean;
  quality_profile_id:number|null; quality_profile_name:string|null;
  effective_quality_profile_id:number|null; effective_quality_profile_name:string|null;
  created_at:string; updated_at:string;
}

export type SeriesSeason = {
  id:number; series_id:number; tmdb_season_id:number|null; season_number:number;
  name:string; overview:string|null; air_date:string|null; poster_path:string|null;
  episode_count:number; monitored:boolean;
  quality_profile_id:number|null; quality_profile_name:string|null;
  effective_quality_profile_id:number|null; effective_quality_profile_name:string|null;
  available_episodes:number; monitored_episodes:number;
  created_at:string; updated_at:string;
}

export type SeasonWithEpisodes = SeriesSeason & { episodes:SeriesEpisode[] }

export type SeriesDetailResponse = {
  series:Series;
  seasons:SeasonWithEpisodes[];
}

export type MediaSearchResult = {
  tmdb_id: number; title: string; original_title: string; year?: number | null;
  overview: string; poster_url?: string | null; backdrop_url?: string | null;
  vote_average: number;
}

export type MovieSearchResult = MediaSearchResult

export type QualityRules = {
  resolutions: Record<string, number>
  sources: Record<string, number>
  codecs: Record<string, number>
  hdr: Record<string, number>
  audio: Record<string, number>
  reject_terms: string[]
  prefer_terms: Record<string, number>
  allow_unknown_resolution: boolean
  allow_unknown_source: boolean
  series_prefer_pack: boolean
  prefer_indexer_priority: boolean
  series_accept_complete: boolean
}

export type QualityProfile = {
  id: number
  name: string
  media_type: 'movie' | 'series'
  enabled: boolean
  upgrade_allowed: boolean
  cutoff_score: number
  min_seeders: number
  min_size_mb: number | null
  max_size_mb: number | null
  max_season_pack_size_mb: number | null
  language_profile_id: number | null
  language_profile_name: string | null
  qbittorrent_category: string
  qbittorrent_tags_template: string
  is_default: boolean
  request_quality: 'standard' | '4k'
  rules: QualityRules
  created_at: string
  updated_at: string
}

export type LanguageProfile = {
  id: number
  name: string
  allowed_languages: string[]
  scores: Record<string, number>
  allow_unknown: boolean
  created_at: string
  updated_at: string
}

export type AutomationState = {
  media_type: 'movie' | 'series'
  media_id: number
  last_search_at: string | null
  last_grab_at: string | null
  last_grab_title: string | null
  last_grab_score: number | null
  last_error: string | null
  status: string
}

export type AutomationStatus = {
  enabled: boolean
  interval_minutes: number
  monitored_movies: number
  monitored_series: number
  wanted_episodes: number
  states: AutomationState[]
  running:boolean
  current_item:string|null
  completed_items:number
  total_items:number
}
export type RssStatus = { enabled:boolean; interval_minutes:number; configured_feeds:number; states:Array<{indexer_id:string;last_synced_at:string|null;last_status:string;last_new_items:number;last_matched_items:number;last_grabbed_items:number;last_error:string|null;updated_at:string}> }

export type Settings = {
  tmdb_api_key_set: boolean;
  tvdb_api_key_set: boolean;
  qbittorrent_host: string;
  qbittorrent_port: number;
  qbittorrent_username: string;
  qbittorrent_password_set: boolean;
  qbittorrent_https: boolean;
  movies_path: string;
  series_path: string;
  downloads_path: string;
  custom_indexers_path: string;
  upstream_indexers_path: string;
  automation_enabled: boolean;
  automation_interval_minutes: number;
  rss_enabled: boolean;
  rss_interval_minutes: number;
  backup_enabled:boolean;
  backup_interval_hours:number;
  backup_retention_count:number;

  import_enabled: boolean;
  import_method: 'auto'|'hardlink'|'copy'|'move'|string;
  rename_enabled: boolean;
  movie_naming_template: string;
  series_naming_template: string;
  keep_reseed_metadata: boolean;
  cleanup_after_seed: boolean;
  torrent_metadata_path: string;
  reseed_path: string;

  ui_theme: 'dark'|'middle'|'light';
  api_enabled:boolean;
  api_key_set:boolean;
  overseerr_compat_enabled:boolean;
}

export type ImportJob = {
  id:number; media_type:'movie'|'series'; media_id:number; release_title:string;
  indexer_id:string; indexer_name:string|null; category:string; qbittorrent_tags:string;
  season_number:number|null; episode_number:number|null; is_season_pack:boolean; qb_hash:string|null;
  status:string; source_path:string|null; library_path:string|null; import_method:string|null;
  file_mappings_json:string; torrent_metadata_path:string|null; last_error:string|null;
  imported_at:string|null; cleaned_at:string|null; reseed_count:number;
  created_at:string; updated_at:string;
}

export type SeedPolicy = {
  indexer_id:string;
  min_seed_time_minutes:number;
  min_ratio:number;
  requirement_mode:'time'|'ratio'|'either'|'both'|'manual';
  cleanup_mode:'remove_torrent_keep_files'|'remove_torrent_and_original'|'manual'|'never';
  updated_at:string;
}

export type TorrentLive = {
  hash:string;
  name:string;
  progress:number;
  dlspeed:number;
  upspeed:number;
  eta:number;
  state:string;
  size:number;
  total_size:number;
  amount_left:number;
  ratio:number;
  num_seeds:number;
  num_leechs:number;
  category:string;
  tags:string;
  tracker:string;
  save_path:string;
  content_path:string;
}

export type DownloadListResponse = {
  status:string;
  torrents:TorrentLive[];
}

export type LibrarySummary = {
  movie_files:number; series_files:number; movie_bytes:number; series_bytes:number; total_bytes:number;
}

export type LibraryScanResult = {
  media_type:'movie'|'series'; root_path:string; scanned_files:number; matched_files:number; unmatched_files:number;
}

export type CalendarEpisode = {
  series_id:number; series_name:string; poster_path:string|null;
  season_number:number; episode_number:number; episode_name:string; air_date:string;
  monitored:boolean; has_file:boolean;
}

export type PublicMediaRequest = {
  id:number; media_type:'movie'|'series'; tmdb_id:number; media_id:number|null;
  quality_profile_id:number|null; monitored:boolean; monitor_mode:string|null;
  requested_by:string|null; created_at:string; updated_at:string;
  status:'pending'|'searching'|'downloading'|'available'|'failed'|string;
  title:string|null;
}

export type QBittorrentCategory = {
  name:string;
  save_path:string;
}

export type QBittorrentCategoryListResponse = {
  status:string;
  categories:QBittorrentCategory[];
}

export type QBittorrentTestResult = {
  status: string; version: string; host: string; latency_ms: number; auth_method: string;
}

export type Diagnostics = {
  version:string; operating_system:string; database:'ok'|'error'|string;
  tmdb_configured:boolean; qbittorrent_configured:boolean;
  enabled_indexers:number; total_indexers:number; automation_enabled:boolean;
  recent_errors:Array<{source:string;message:string;occurred_at:string}>;
}

export type DatabaseBackup = { filename:string; size_bytes:number; created_at:number }
export type BackupList = { directory:string; backups:DatabaseBackup[] }

async function parseError(response: Response): Promise<string> {
  const text = await response.text()
  return text || `HTTP ${response.status}`
}

async function json<T>(url: string, init?: RequestInit): Promise<T> {
  const res = await fetch(url, init)
  if (!res.ok) throw new Error(await parseError(res))
  return res.json()
}

export const getMovies = () => json<Movie[]>('/api/movies')
export const searchMovies = (query: string) => json<MediaSearchResult[]>(`/api/movies/search?query=${encodeURIComponent(query)}`)
export const addMovie = (tmdbId: number, qualityProfileId?: number | null) => json<Movie>('/api/movies', {
  method:'POST', headers:{'Content-Type':'application/json'},
  body: JSON.stringify({tmdb_id:tmdbId, monitored:true, library_path:null, quality_profile_id: qualityProfileId ?? null})
})
export async function updateMovie(id:number, payload:boolean|{monitored?:boolean;quality_profile_id?:number;library_path?:string}) {
  const body = typeof payload === 'boolean' ? {monitored:payload} : payload
  return json<Movie>(`/api/movies/${id}`, {
    method:'PUT', headers:{'Content-Type':'application/json'}, body: JSON.stringify(body)
  })
}
export async function deleteMovie(id:number) {
  const res = await fetch(`/api/movies/${id}`, {method:'DELETE'})
  if (!res.ok) throw new Error(await parseError(res))
}

export const getSeries = () => json<Series[]>('/api/series')
export const searchSeries = (query: string) => json<MediaSearchResult[]>(`/api/series/search?query=${encodeURIComponent(query)}`)
export const addSeries = (tmdbId: number, qualityProfileId?: number | null, monitorMode='all') => json<Series>('/api/series', {
  method:'POST', headers:{'Content-Type':'application/json'},
  body: JSON.stringify({tmdb_id:tmdbId, monitored:true, library_path:null, quality_profile_id: qualityProfileId ?? null, monitor_mode:monitorMode})
})
export const getSeriesDetail = (id:number) => json<SeriesDetailResponse>(`/api/series/${id}`)
export const getSeriesSeasons = (id:number) => json<SeasonWithEpisodes[]>(`/api/series/${id}/seasons`)
export const refreshSeries = (id:number) => json<SeriesDetailResponse>(`/api/series/${id}/refresh`,{method:'POST'})
export const updateSeries = (id:number, payload:boolean|{monitored?:boolean;quality_profile_id?:number;library_path?:string;monitor_mode?:string}) => json<Series>(`/api/series/${id}`, {
  method:'PUT', headers:{'Content-Type':'application/json'}, body:JSON.stringify(typeof payload === 'boolean' ? {monitored:payload} : payload)
})
export async function deleteSeries(id:number) {
  const res = await fetch(`/api/series/${id}`, {method:'DELETE'})
  if (!res.ok) throw new Error(await parseError(res))
}

export const getQualityProfiles = (mediaType?:'movie'|'series') => json<QualityProfile[]>(`/api/profiles${mediaType?`?media_type=${mediaType}`:''}`)
export const createQualityProfile = (payload:Omit<QualityProfile,'id'|'language_profile_name'|'is_default'|'created_at'|'updated_at'>) => json<QualityProfile>('/api/profiles', {
  method:'POST', headers:{'Content-Type':'application/json'}, body:JSON.stringify(payload)
})
export const saveQualityProfile = (id:number,payload:Omit<QualityProfile,'id'|'language_profile_name'|'is_default'|'created_at'|'updated_at'>) => json<QualityProfile>(`/api/profiles/${id}`, {
  method:'PUT', headers:{'Content-Type':'application/json'}, body:JSON.stringify(payload)
})
export async function deleteQualityProfile(id:number){
  const res=await fetch(`/api/profiles/${id}`,{method:'DELETE'}); if(!res.ok)throw new Error(await parseError(res))
}
export const setDefaultQualityProfile = (id:number) => json<QualityProfile>(`/api/profiles/${id}/default`,{method:'PUT'})

export const getLanguageProfiles = () => json<LanguageProfile[]>('/api/language-profiles')
export const createLanguageProfile = (payload:Omit<LanguageProfile,'id'|'created_at'|'updated_at'>) => json<LanguageProfile>('/api/language-profiles', {
  method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(payload)
})
export const saveLanguageProfile = (id:number,payload:Omit<LanguageProfile,'id'|'created_at'|'updated_at'>) => json<LanguageProfile>(`/api/language-profiles/${id}`, {
  method:'PUT',headers:{'Content-Type':'application/json'},body:JSON.stringify(payload)
})
export async function deleteLanguageProfile(id:number){
  const res=await fetch(`/api/language-profiles/${id}`,{method:'DELETE'}); if(!res.ok)throw new Error(await parseError(res))
}

export const getAutomationStatus = () => json<AutomationStatus>('/api/automation/status')
export const runAutomationNow = () => json<{searched:number;grabbed:number;skipped:number;errors:number;reconciled_missing:number}>('/api/automation/run',{method:'POST'})
export const getRssStatus = () => json<RssStatus>('/api/rss/status')
export const runRssNow = () => json<{feeds:number;new_items:number;matched:number;grabbed:number;skipped:number;errors:number}>('/api/rss/run',{method:'POST'})

export const getSettings = () => json<Settings>('/api/settings')
export const getBackups = () => json<BackupList>('/api/backups')
export const createBackup = () => json<{backup:DatabaseBackup;message:string}>('/api/backups',{method:'POST'})
export const restoreBackup = (filename:string) => json<{message:string}>(`/api/backups/${encodeURIComponent(filename)}/restore`,{method:'POST'})
export const deleteBackup = async (filename:string) => { const res=await fetch(`/api/backups/${encodeURIComponent(filename)}`,{method:'DELETE'}); if(!res.ok) throw new Error(await parseError(res)); return res.json() as Promise<{message:string}> }
export const uploadBackup = async (file:File) => { const res=await fetch('/api/backups/upload',{method:'POST',headers:{'Content-Type':'application/vnd.sqlite3'},body:file}); if(!res.ok) throw new Error(await parseError(res)); return res.json() as Promise<{backup:DatabaseBackup;message:string}> }
export const backupDownloadUrl = (filename:string) => `/api/backups/${encodeURIComponent(filename)}/download`
export async function saveSettings(payload: Record<string, unknown>) {
  const res = await fetch('/api/settings', {
    method:'PUT', headers:{'Content-Type':'application/json'}, body:JSON.stringify(payload)
  })
  if (!res.ok) throw new Error(await parseError(res))
}
export const testQBittorrent = () => json<QBittorrentTestResult>('/api/settings/test-qbittorrent', {method:'POST'})
export const getDiagnostics = () => json<Diagnostics>('/api/diagnostics')


export const getImports = () => json<ImportJob[]>('/api/imports')
export const reseedImport = (id:number) => json<{status:string;path:string}>(`/api/imports/${id}/reseed`,{method:'POST'})
export const deleteImports = (ids:number[]) => json<{deleted:number}>('/api/imports',{method:'DELETE',headers:{'Content-Type':'application/json'},body:JSON.stringify({ids})})
export const getSeedPolicies = () => json<SeedPolicy[]>('/api/seed-policies')
export const saveSeedPolicy = (indexerId:string,payload:Omit<SeedPolicy,'indexer_id'|'updated_at'>) =>
  json<SeedPolicy>(`/api/seed-policies/${encodeURIComponent(indexerId)}`,{
    method:'PUT',headers:{'Content-Type':'application/json'},body:JSON.stringify(payload)
  })

export const getDownloadsLive = () => json<DownloadListResponse>('/api/downloads')
export const updateTorrentOrganization = (hash:string,payload:{category:string;tags:string;previous_tags:string}) => json<{status:string}>(`/api/downloads/${encodeURIComponent(hash)}/organization`,{method:'PUT',headers:{'Content-Type':'application/json'},body:JSON.stringify(payload)})

export const getQBittorrentCategories = () => json<QBittorrentCategoryListResponse>('/api/qbittorrent/categories')
export const createQBittorrentCategory = (name:string, save_path:string) => json<{status:string}>('/api/qbittorrent/categories',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({name,save_path})})
export const updateQBittorrentCategory = (name:string, save_path:string) => json<{status:string}>(`/api/qbittorrent/categories/${encodeURIComponent(name)}`,{method:'PUT',headers:{'Content-Type':'application/json'},body:JSON.stringify({name,save_path})})
export const deleteQBittorrentCategory = (name:string) => json<{status:string}>(`/api/qbittorrent/categories/${encodeURIComponent(name)}`,{method:'DELETE'})
export const getQBittorrentTags = () => json<{status:string;tags:string[]}>('/api/qbittorrent/tags')
export type DirectoryListing = {path:string;parent:string|null;roots:string[];directories:string[]}
export const getDirectories = (path?:string) => json<DirectoryListing>(`/api/filesystem/directories${path?`?path=${encodeURIComponent(path)}`:''}`)
export const createQBittorrentTags = (tags:string[]) => json<{status:string}>('/api/qbittorrent/tags',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({tags})})
export const deleteQBittorrentTags = (tags:string[]) => json<{status:string}>('/api/qbittorrent/tags',{method:'DELETE',headers:{'Content-Type':'application/json'},body:JSON.stringify({tags})})

export const updateSeriesSeason = (
  seriesId:number, seasonNumber:number,
  payload:{monitored?:boolean;quality_profile_id?:number|null;clear_quality_profile?:boolean}
) => json<SeasonWithEpisodes>(`/api/series/${seriesId}/seasons/${seasonNumber}`,{
  method:'PUT',headers:{'Content-Type':'application/json'},body:JSON.stringify(payload)
})
export const updateSeriesEpisode = (
  seriesId:number, episodeId:number,
  payload:{monitored?:boolean;quality_profile_id?:number|null;clear_quality_profile?:boolean}
) => json<SeriesEpisode>(`/api/series/${seriesId}/episodes/${episodeId}`,{
  method:'PUT',headers:{'Content-Type':'application/json'},body:JSON.stringify(payload)
})

export const rescanLibrary = (mediaType:'movie'|'series'|'all'='all') => json<LibraryScanResult[]>(`/api/library/rescan?media_type=${mediaType}`,{method:'POST'})
export const getCalendar = (days=45,includePast=true) => json<CalendarEpisode[]>(`/api/calendar?days=${days}&include_past=${includePast}`)

export const getLibrarySummary = () => json<LibrarySummary>('/api/library/summary')

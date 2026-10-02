import { FormEvent, useCallback, useEffect, useMemo, useRef, useState } from 'react'
import './Downloads.css'
import { QBittorrentCategory, createQBittorrentCategory, createQBittorrentTags, deleteQBittorrentCategory, deleteQBittorrentTags, getQBittorrentCategories, getQBittorrentTags, getRejectedReleases, RejectedRelease, rejectTorrent, unrejectRelease, updateQBittorrentCategory, updateTorrentOrganization } from '../lib/api'
import { FolderPicker } from '../components/FolderPicker'

type Torrent = {
  hash: string
  name: string

  size: number
  total_size: number
  amount_left: number
  progress: number
  availability: number

  dlspeed: number
  upspeed: number
  dl_limit: number
  up_limit: number
  downloaded: number
  downloaded_session: number
  uploaded: number
  uploaded_session: number
  eta: number
  ratio: number
  ratio_limit: number

  num_seeds: number
  num_complete: number
  num_leechs: number
  num_incomplete: number

  state: string
  priority: number
  force_start: boolean
  seq_dl: boolean
  f_l_piece_prio: boolean
  super_seeding: boolean

  category: string
  tags: string
  tracker: string

  save_path: string
  content_path: string
  magnet_uri: string

  added_on: number
  completion_on: number
  last_activity: number
  time_active: number
  seeding_time: number

  // Set when Oberiz itself picked and downloaded this torrent.
  oberiz_job_id?: number | null
}

type DownloadListResponse = {
  status: string
  torrents: Torrent[]
}

type Filter = 'all' | 'downloading' | 'seeding' | 'completed'
type SortKey =
  | 'name'
  | 'progress'
  | 'status'
  | 'dlspeed'
  | 'upspeed'
  | 'eta'
  | 'size'
  | 'seeds'
  | 'peers'
  | 'ratio'
  | 'category'
  | 'tags'
  | 'tracker'
  | 'availability'
  | 'added'

type SortDirection = 'asc' | 'desc'

type SpeedPoint = {
  down: number
  up: number
}

function formatBytes(bytes: number, decimals = 1) {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB']
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1)
  return `${(bytes / 1024 ** index).toFixed(index === 0 ? 0 : decimals)} ${units[index]}`
}

function formatSpeed(bytesPerSecond: number) {
  return `${formatBytes(Math.max(0, bytesPerSecond))}/s`
}

function formatEta(seconds: number) {
  if (!Number.isFinite(seconds) || seconds < 0 || seconds >= 8_640_000) return '∞'
  if (seconds === 0) return '—'

  const days = Math.floor(seconds / 86_400)
  const hours = Math.floor((seconds % 86_400) / 3_600)
  const minutes = Math.floor((seconds % 3_600) / 60)

  if (days > 0) return `${days}d ${hours}h`
  if (hours > 0) return `${hours}h ${minutes}m`
  return `${Math.max(1, minutes)}m`
}

function relativeTime(timestamp: number) {
  if (!timestamp) return '—'
  const seconds = Math.max(0, Math.floor(Date.now() / 1000 - timestamp))

  if (seconds < 60) return 'just now'
  if (seconds < 3600) return `${Math.floor(seconds / 60)} min ago`
  if (seconds < 86400) return `${Math.floor(seconds / 3600)} h ago`
  return `${Math.floor(seconds / 86400)} d ago`
}

function formatRatio(ratio: number) {
  if (!Number.isFinite(ratio)) return '—'
  return ratio.toFixed(2)
}

function formatAvailability(value: number) {
  if (!Number.isFinite(value) || value < 0) return '—'
  return value.toFixed(value >= 10 ? 0 : 2)
}

function normalizeSortText(value: string) {
  return (value || '').trim().toLocaleLowerCase()
}

function stateInfo(state: string) {
  const value = state.toLowerCase()

  if (value.includes('downloading') || value.includes('forceddl') || value === 'metadl') {
    return { label: 'Downloading', kind: 'downloading', active: true }
  }
  if (value.includes('uploading') || value.includes('forcedup') || value.includes('stalledup')) {
    return { label: 'Seeding', kind: 'seeding', active: true }
  }
  if (value.includes('stalleddl')) {
    return { label: 'Stalled', kind: 'warning', active: true }
  }
  if (value.includes('paused') || value.includes('stopped')) {
    return { label: 'Stopped', kind: 'stopped', active: false }
  }
  if (value.includes('checking')) {
    return { label: 'Checking', kind: 'checking', active: true }
  }
  if (value.includes('error') || value.includes('missingfiles')) {
    return { label: 'Error', kind: 'error', active: false }
  }
  if (value.includes('queued') || value.includes('allocating')) {
    return { label: 'Queued', kind: 'queued', active: true }
  }
  if (value.includes('moving')) {
    return { label: 'Moving', kind: 'checking', active: true }
  }
  if (value.includes('stalled')) {
    return { label: 'Stalled', kind: 'warning', active: true }
  }

  return { label: state || 'Unknown', kind: 'stopped', active: false }
}

function trackerName(tracker: string) {
  if (!tracker) return '—'

  try {
    return new URL(tracker).hostname.replace(/^www\./, '')
  } catch {
    return tracker
  }
}

function sourceFromTags(tags: string) {
  if (!tags) return ''

  const source = tags
    .split(',')
    .map((tag) => tag.trim())
    .find((tag) => tag.toLowerCase().startsWith('source:'))

  return source ? source.slice('source:'.length).trim() : ''
}

function torrentSource(torrent: Torrent) {
  const tracker = trackerName(torrent.tracker)
  const source = sourceFromTags(torrent.tags)

  return {
    primary: tracker !== '—' ? tracker : source || '—',
    secondary: source && source !== tracker ? source : 'qBittorrent',
  }
}

function todayStartUnix() {
  const date = new Date()
  date.setHours(0, 0, 0, 0)
  return Math.floor(date.getTime() / 1000)
}

function Sparkline({ points, field }: { points: SpeedPoint[]; field: 'down' | 'up' }) {
  const values = points.map((point) => point[field])
  const max = Math.max(1, ...values)
  const width = 300
  const height = 72

  const path = values
    .map((value, index) => {
      const x = values.length <= 1 ? 0 : (index / (values.length - 1)) * width
      const y = height - (value / max) * (height - 8) - 4
      return `${x.toFixed(1)},${y.toFixed(1)}`
    })
    .join(' ')

  return (
    <svg className={`speed-line speed-line-${field}`} viewBox={`0 0 ${width} ${height}`} preserveAspectRatio="none">
      <polyline points={path || `0,${height} ${width},${height}`} />
    </svg>
  )
}

export function Downloads() {
  const [torrents, setTorrents] = useState<Torrent[]>([])
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null)
  const [filter, setFilter] = useState<Filter>('all')
  const [query, setQuery] = useState('')
  const [sortKey, setSortKey] = useState<SortKey>('added')
  const [sortDirection, setSortDirection] = useState<SortDirection>('desc')
  const [magnetOpen, setMagnetOpen] = useState(false)
  const [magnet, setMagnet] = useState('')
  const [category, setCategory] = useState('')
  const [busyHash, setBusyHash] = useState<string | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<Torrent | null>(null)
  const [deleteFiles, setDeleteFiles] = useState(false)
  // Same confirmation dialog as delete, but also records the release as rejected.
  const [rejectMode, setRejectMode] = useState(false)
  const [rejectedReleases, setRejectedReleases] = useState<RejectedRelease[]>([])
  const [actionError, setActionError] = useState<string | null>(null)
  const [organizationOpen, setOrganizationOpen] = useState(false)
  const [categories, setCategories] = useState<QBittorrentCategory[]>([])
  const [tags, setTags] = useState<string[]>([])
  const [categoryName, setCategoryName] = useState('')
  const [categoryPath, setCategoryPath] = useState('')
  const [categoryPaths, setCategoryPaths] = useState<Record<string,string>>({})
  const [tagName, setTagName] = useState('')
  const [organizationBusy, setOrganizationBusy] = useState(false)
  const [editingOrganization,setEditingOrganization]=useState<string|null>(null)
  const [editCategory,setEditCategory]=useState('')
  const [editTags,setEditTags]=useState('')
  const [speedHistory, setSpeedHistory] = useState<SpeedPoint[]>(
    Array.from({ length: 30 }, () => ({ down: 0, up: 0 })),
  )
  const mounted = useRef(true)

  const totals = useMemo(() => {
    const result = {
      downloading: 0,
      seeding: 0,
      completed: 0,
      completedToday: 0,
      completedTodayBytes: 0,
      queued: 0,
      stalled: 0,
      downloadSpeed: 0,
      uploadSpeed: 0,
    }

    const today = todayStartUnix()

    for (const torrent of torrents) {
      const state = stateInfo(torrent.state)
      result.downloadSpeed += torrent.dlspeed || 0
      result.uploadSpeed += torrent.upspeed || 0

      if (state.kind === 'downloading') result.downloading += 1
      if (state.kind === 'seeding') result.seeding += 1
      if (state.kind === 'queued') result.queued += 1
      if (state.kind === 'warning') result.stalled += 1

      if (torrent.progress >= 0.999999) {
        result.completed += 1
        if (torrent.completion_on >= today) {
          result.completedToday += 1
          result.completedTodayBytes += torrent.size || 0
        }
      }
    }

    return result
  }, [torrents])

  const loadDownloads = useCallback(async (silent = false) => {
    if (silent) setRefreshing(true)
    else setLoading(true)

    try {
      const response = await fetch('/api/downloads', {
        headers: { Accept: 'application/json' },
      })

      if (!response.ok) {
        const text = await response.text()
        throw new Error(text || `HTTP ${response.status}`)
      }

      const data = (await response.json()) as DownloadListResponse
      const next = Array.isArray(data.torrents) ? data.torrents : []

      if (!mounted.current) return

      setTorrents(next)
      setError(null)
      setLastUpdated(new Date())

      const down = next.reduce((sum, item) => sum + (item.dlspeed || 0), 0)
      const up = next.reduce((sum, item) => sum + (item.upspeed || 0), 0)

      setSpeedHistory((history) => [...history.slice(-29), { down, up }])
    } catch (err) {
      if (mounted.current) {
        setError(err instanceof Error ? err.message : 'Could not load downloads')
      }
    } finally {
      if (mounted.current) {
        setLoading(false)
        setRefreshing(false)
      }
    }
  }, [])

  const loadOrganization = useCallback(async () => {
    try {
      const [categoryResult, tagResult] = await Promise.all([getQBittorrentCategories(), getQBittorrentTags()])
      if (!mounted.current) return
      setCategories(categoryResult.categories)
      setTags(tagResult.tags)
      setCategoryPaths(Object.fromEntries(categoryResult.categories.map(item=>[item.name,item.save_path])))
    } catch (err) {
      if (mounted.current) setActionError(err instanceof Error ? err.message : 'Could not load qBittorrent organization')
    }
  }, [])

  const loadRejected = useCallback(async () => {
    try {
      const rows = await getRejectedReleases()
      if (mounted.current) setRejectedReleases(rows)
    } catch {
      // The list is secondary to the downloads table; a failed refresh keeps the last one.
    }
  }, [])

  async function undoRejection(id: number) {
    setActionError(null)
    try {
      await unrejectRelease(id)
      await loadRejected()
    } catch (err) {
      setActionError(err instanceof Error ? err.message : 'Could not undo the rejection')
    }
  }

  useEffect(() => {
    mounted.current = true
    void loadDownloads()
    void loadOrganization()
    void loadRejected()

    let timer = 0

    const schedule = () => {
      window.clearInterval(timer)
      const interval = document.visibilityState === 'visible' ? 2000 : 10000
      timer = window.setInterval(() => {
        void loadDownloads(true)
      }, interval)
    }

    const handleVisibility = () => {
      schedule()
      if (document.visibilityState === 'visible') {
        void loadDownloads(true)
      }
    }

    schedule()
    document.addEventListener('visibilitychange', handleVisibility)

    return () => {
      mounted.current = false
      document.removeEventListener('visibilitychange', handleVisibility)
      window.clearInterval(timer)
    }
  }, [loadDownloads, loadOrganization, loadRejected])

  const visibleTorrents = useMemo(() => {
    const needle = query.trim().toLowerCase()

    const filtered = torrents.filter((torrent) => {
      const state = stateInfo(torrent.state)

      const filterMatches =
        filter === 'all' ||
        (filter === 'downloading' && (state.kind === 'downloading' || state.kind === 'warning')) ||
        (filter === 'seeding' && state.kind === 'seeding') ||
        (filter === 'completed' && torrent.progress >= 0.999999)

      const queryMatches =
        !needle ||
        torrent.name.toLowerCase().includes(needle) ||
        torrent.category.toLowerCase().includes(needle) ||
        torrent.tags.toLowerCase().includes(needle) ||
        trackerName(torrent.tracker).toLowerCase().includes(needle) ||
        sourceFromTags(torrent.tags).toLowerCase().includes(needle) ||
        torrent.save_path.toLowerCase().includes(needle)

      return filterMatches && queryMatches
    })

    const direction = sortDirection === 'asc' ? 1 : -1

    return [...filtered].sort((a, b) => {
      const aState = stateInfo(a.state).label
      const bState = stateInfo(b.state).label

      const numeric = (left: number, right: number) =>
        ((Number.isFinite(left) ? left : 0) - (Number.isFinite(right) ? right : 0)) * direction

      const textual = (left: string, right: string) =>
        normalizeSortText(left).localeCompare(normalizeSortText(right)) * direction

      switch (sortKey) {
        case 'name': return textual(a.name, b.name)
        case 'progress': return numeric(a.progress, b.progress)
        case 'status': return textual(aState, bState)
        case 'dlspeed': return numeric(a.dlspeed, b.dlspeed)
        case 'upspeed': return numeric(a.upspeed, b.upspeed)
        case 'eta': return numeric(a.eta, b.eta)
        case 'size': return numeric(a.size, b.size)
        case 'seeds': return numeric(a.num_seeds, b.num_seeds)
        case 'peers': return numeric(a.num_leechs, b.num_leechs)
        case 'ratio': return numeric(a.ratio, b.ratio)
        case 'category': return textual(a.category, b.category)
        case 'tags': return textual(a.tags, b.tags)
        case 'tracker': return textual(torrentSource(a).primary, torrentSource(b).primary)
        case 'availability': return numeric(a.availability, b.availability)
        case 'added': return numeric(a.added_on, b.added_on)
        default: return 0
      }
    })
  }, [filter, query, sortDirection, sortKey, torrents])

  const recentCompleted = useMemo(
    () =>
      torrents
        .filter((torrent) => torrent.progress >= 0.999999 && torrent.completion_on > 0)
        .sort((a, b) => b.completion_on - a.completion_on)
        .slice(0, 4),
    [torrents],
  )

  const action = useCallback(async (url: string, options?: RequestInit) => {
    const response = await fetch(url, options)

    if (!response.ok) {
      const text = await response.text()
      throw new Error(text || `HTTP ${response.status}`)
    }
  }, [])

  async function toggleTorrent(torrent: Torrent) {
    const state = stateInfo(torrent.state)
    setBusyHash(torrent.hash)
    setActionError(null)

    try {
      await action(
        `/api/downloads/${torrent.hash}/${state.active ? 'stop' : 'start'}`,
        { method: 'POST' },
      )
      await loadDownloads(true)
    } catch (err) {
      setActionError(err instanceof Error ? err.message : 'Action failed')
    } finally {
      setBusyHash(null)
    }
  }

  async function submitMagnet(event: FormEvent) {
    event.preventDefault()
    setActionError(null)

    try {
      await action('/api/downloads/magnet', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          url: magnet.trim(),
          category: category.trim(),
        }),
      })

      setMagnet('')
      setCategory('')
      setMagnetOpen(false)
      await loadDownloads(true)
    } catch (err) {
      setActionError(err instanceof Error ? err.message : 'Could not add magnet')
    }
  }

  async function confirmDelete() {
    if (!deleteTarget) return

    setBusyHash(deleteTarget.hash)
    setActionError(null)

    try {
      if (rejectMode) {
        await rejectTorrent(deleteTarget.hash, deleteFiles)
      } else {
        await action(
          `/api/downloads/${deleteTarget.hash}?delete_files=${deleteFiles ? 'true' : 'false'}`,
          { method: 'DELETE' },
        )
      }

      setDeleteTarget(null)
      setDeleteFiles(false)
      setRejectMode(false)
      await loadDownloads(true)
      void loadRejected()
    } catch (err) {
      setActionError(err instanceof Error ? err.message : rejectMode ? 'Could not reject release' : 'Could not delete torrent')
    } finally {
      setBusyHash(null)
    }
  }

  async function createCategory(event: FormEvent) {
    event.preventDefault()
    setOrganizationBusy(true); setActionError(null)
    try {
      await createQBittorrentCategory(categoryName.trim(),categoryPath.trim())
      setCategoryName(''); setCategoryPath('')
      await loadOrganization()
    } catch (err) { setActionError(err instanceof Error ? err.message : 'Could not create category') }
    finally { setOrganizationBusy(false) }
  }
  async function saveCategoryPath(name:string) {
    setOrganizationBusy(true); setActionError(null)
    try { await updateQBittorrentCategory(name,categoryPaths[name]??''); await loadOrganization() }
    catch (err) { setActionError(err instanceof Error ? err.message : 'Could not update category') }
    finally { setOrganizationBusy(false) }
  }
  async function removeCategory(name:string) {
    if (!confirm(`Remove the category “${name}”? Torrents will remain in qBittorrent but become uncategorized.`)) return
    setOrganizationBusy(true); setActionError(null)
    try { await deleteQBittorrentCategory(name); await loadOrganization() }
    catch (err) { setActionError(err instanceof Error ? err.message : 'Could not remove category') }
    finally { setOrganizationBusy(false) }
  }
  async function createTag(event: FormEvent) {
    event.preventDefault()
    setOrganizationBusy(true); setActionError(null)
    try { await createQBittorrentTags([tagName.trim()]); setTagName(''); await loadOrganization() }
    catch (err) { setActionError(err instanceof Error ? err.message : 'Could not create tag') }
    finally { setOrganizationBusy(false) }
  }
  async function removeTag(tag:string) {
    if (!confirm(`Remove the tag “${tag}” from qBittorrent?`)) return
    setOrganizationBusy(true); setActionError(null)
    try { await deleteQBittorrentTags([tag]); await loadOrganization() }
    catch (err) { setActionError(err instanceof Error ? err.message : 'Could not remove tag') }
    finally { setOrganizationBusy(false) }
  }
  function beginOrganizationEdit(torrent:Torrent){setEditingOrganization(torrent.hash);setEditCategory(torrent.category);setEditTags(torrent.tags)}
  async function saveTorrentOrganization(torrent:Torrent){
    setBusyHash(torrent.hash);setActionError(null)
    try{await updateTorrentOrganization(torrent.hash,{category:editCategory,tags:editTags,previous_tags:torrent.tags});setEditingOrganization(null);await loadDownloads(true);await loadOrganization()}
    catch(err){setActionError(err instanceof Error?err.message:'Could not update torrent organization')}
    finally{setBusyHash(null)}
  }

  const totalCombinedSpeed = totals.downloadSpeed + totals.uploadSpeed
  const queueHealth = error
    ? 0
    : torrents.length === 0
      ? 100
      : Math.max(0, Math.round(100 - ((totals.stalled + totals.queued) / torrents.length) * 100))

  return (
    <div className="dl-page">
      <header className="dl-heading">
        <div>
          <p className="dl-kicker">DOWNLOAD CLIENT</p>
          <h1>Downloads</h1>
          <p>Live torrent activity from qBittorrent.</p>
        </div>

        <button className="dl-primary" type="button" onClick={() => setMagnetOpen(true)}>
          <span>＋</span>
          Add Magnet
        </button>
      </header>

      {actionError && (
        <div className="dl-alert">
          <span>{actionError}</span>
          <button type="button" onClick={() => setActionError(null)} aria-label="Dismiss">×</button>
        </div>
      )}

      <section className="dl-stats">
        <article className="dl-stat">
          <div className="dl-stat-icon down">↓</div>
          <div>
            <span>Active Downloads</span>
            <strong>{totals.downloading}</strong>
            <small>↓ {formatSpeed(totals.downloadSpeed)} total</small>
          </div>
        </article>

        <article className="dl-stat">
          <div className="dl-stat-icon up">↑</div>
          <div>
            <span>Seeding</span>
            <strong>{totals.seeding}</strong>
            <small>↑ {formatSpeed(totals.uploadSpeed)} total</small>
          </div>
        </article>

        <article className="dl-stat">
          <div className="dl-stat-icon done">✓</div>
          <div>
            <span>Completed Today</span>
            <strong>{totals.completedToday}</strong>
            <small>{formatBytes(totals.completedTodayBytes)} added</small>
          </div>
        </article>

        <article className="dl-stat">
          <div className="dl-stat-icon speed">⌁</div>
          <div>
            <span>Total Speed</span>
            <strong>{formatSpeed(totalCombinedSpeed)}</strong>
            <small>↓ {formatSpeed(totals.downloadSpeed)} · ↑ {formatSpeed(totals.uploadSpeed)}</small>
          </div>
        </article>
      </section>

      <section className="dl-organization">
        <button className="dl-organization-toggle" type="button" onClick={()=>setOrganizationOpen(open=>!open)} aria-expanded={organizationOpen}>
          <span><b>qBittorrent Organization</b><small>Manage categories, save paths and tags</small></span><i>{organizationOpen?'−':'+'}</i>
        </button>
        {organizationOpen&&<div className="dl-organization-body">
          <div className="dl-category-manager">
            <header><div><strong>Categories</strong><small>Each category can have its own default save path.</small></div><span>{categories.length} total</span></header>
            <form className="dl-category-create" onSubmit={createCategory}>
              <input required value={categoryName} onChange={event=>setCategoryName(event.target.value)} placeholder="Category name"/>
              <input value={categoryPath} onChange={event=>setCategoryPath(event.target.value)} placeholder="Save path (optional)"/><FolderPicker value={categoryPath} onChange={setCategoryPath} label="Browse"/>
              <button className="dl-primary" disabled={organizationBusy}>Create</button>
            </form>
            <div className="dl-category-list">
              {categories.map(item=><article key={item.name}><strong>{item.name}</strong><input value={categoryPaths[item.name]??''} onChange={event=>setCategoryPaths(paths=>({...paths,[item.name]:event.target.value}))} placeholder="Default save path"/><FolderPicker value={categoryPaths[item.name]??''} onChange={path=>setCategoryPaths(paths=>({...paths,[item.name]:path}))} label="Browse"/><button type="button" disabled={organizationBusy} onClick={()=>void saveCategoryPath(item.name)}>Save</button><button type="button" className="danger" disabled={organizationBusy} onClick={()=>void removeCategory(item.name)}>Remove</button></article>)}
              {!categories.length&&<p>No categories configured in qBittorrent.</p>}
            </div>
          </div>
          <div className="dl-tag-manager">
            <header><div><strong>Tags</strong><small>Create global qBittorrent tags for manual organization.</small></div><span>{tags.length} total</span></header>
            <form onSubmit={createTag}><input required value={tagName} onChange={event=>setTagName(event.target.value)} placeholder="New tag"/><button className="dl-primary" disabled={organizationBusy}>Create</button></form>
            <div className="dl-tag-list">{tags.map(tag=><span key={tag}>{tag}<button type="button" disabled={organizationBusy} onClick={()=>void removeTag(tag)} aria-label={`Remove ${tag}`}>×</button></span>)}{!tags.length&&<p>No tags configured in qBittorrent.</p>}</div>
          </div>
        </div>}
      </section>

      <section className="dl-panel">
        <div className="dl-toolbar">
          <div className="dl-toolbar-title">
            <span className="dl-download-mark">↓</span>
            <strong>Downloads ({torrents.length})</strong>
          </div>

          <div className="dl-filters" role="tablist" aria-label="Download filters">
            <button className={filter === 'all' ? 'active' : ''} onClick={() => setFilter('all')}>
              All ({torrents.length})
            </button>
            <button className={filter === 'downloading' ? 'active' : ''} onClick={() => setFilter('downloading')}>
              Downloading ({totals.downloading})
            </button>
            <button className={filter === 'seeding' ? 'active' : ''} onClick={() => setFilter('seeding')}>
              Seeding ({totals.seeding})
            </button>
            <button className={filter === 'completed' ? 'active' : ''} onClick={() => setFilter('completed')}>
              Completed ({totals.completed})
            </button>
          </div>

          <div className="dl-sort-wrap">
            <span>Sort</span>
            <select
              value={sortKey}
              onChange={(event) => setSortKey(event.target.value as SortKey)}
              aria-label="Sort downloads by"
            >
              <option value="added">Added</option>
              <option value="name">Name</option>
              <option value="progress">Progress</option>
              <option value="status">Status</option>
              <option value="dlspeed">Download speed</option>
              <option value="upspeed">Upload speed</option>
              <option value="eta">ETA</option>
              <option value="size">Size</option>
              <option value="seeds">Seeds</option>
              <option value="peers">Peers</option>
              <option value="ratio">Ratio</option>
              <option value="category">Category</option>
              <option value="tags">Tags</option>
              <option value="tracker">Tracker / source</option>
              <option value="availability">Availability</option>
            </select>
            <button
              className="dl-sort-direction"
              type="button"
              onClick={() => setSortDirection((current) => current === 'asc' ? 'desc' : 'asc')}
              title={sortDirection === 'asc' ? 'Ascending' : 'Descending'}
              aria-label={sortDirection === 'asc' ? 'Ascending sort' : 'Descending sort'}
            >
              {sortDirection === 'asc' ? '↑' : '↓'}
            </button>
          </div>

          <div className="dl-search-wrap">
            <span>⌕</span>
            <input
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Search downloads..."
              aria-label="Search downloads"
            />
          </div>

          <button
            className="dl-icon-button"
            type="button"
            onClick={() => void loadDownloads(true)}
            disabled={refreshing}
            title="Refresh"
          >
            {refreshing ? '↻' : '⟳'}
          </button>
        </div>

        {error && (
          <div className="dl-connection-error">
            <strong>qBittorrent connection error</strong>
            <span>{error}</span>
          </div>
        )}

        {loading && torrents.length === 0 ? (
          <div className="dl-empty">
            <div className="dl-spinner" />
            <h3>Loading qBittorrent…</h3>
          </div>
        ) : visibleTorrents.length === 0 ? (
          <div className="dl-empty">
            <div className="dl-empty-icon">↓</div>
            <h3>{torrents.length === 0 ? 'No torrents yet' : 'No matching downloads'}</h3>
            <p>
              {torrents.length === 0
                ? 'qBittorrent is connected. Add a magnet or connect your main qBittorrent later.'
                : 'Try another filter or search term.'}
            </p>
          </div>
        ) : (
          <div className="dl-table-wrap">
            <table className="dl-table">
              <thead>
                <tr>
                  <th>Title</th>
                  <th>Progress</th>
                  <th>Status</th>
                  <th>Seeds / Peers</th>
                  <th>Speed</th>
                  <th>ETA</th>
                  <th>Size</th>
                  <th>Ratio</th>
                  <th>Category / Tags</th>
                  <th>Tracker</th>
                  <th>Availability</th>
                  <th>Added</th>
                  <th className="dl-actions-head">Actions</th>
                </tr>
              </thead>

              <tbody>
                {visibleTorrents.map((torrent) => {
                  const progress = Math.min(100, Math.max(0, (torrent.progress || 0) * 100))
                  const state = stateInfo(torrent.state)
                  const busy = busyHash === torrent.hash

                  return (
                    <tr key={torrent.hash}>
                      <td className="dl-title-cell" data-label="Title">
                        <div className="dl-media-thumb">{torrent.name.slice(0, 1).toUpperCase() || '↓'}</div>
                        <div className="dl-media-info">
                          <strong title={torrent.name}>{torrent.name || 'Unnamed torrent'}</strong>
                          <span>
                            {torrent.category || 'Torrent'}
                            {torrent.tags ? ` · ${torrent.tags}` : ''}
                          </span>
                        </div>
                      </td>

                      <td data-label="Progress">
                        <div className="dl-progress-cell">
                          <div className="dl-progress">
                            <i style={{ width: `${progress}%` }} />
                          </div>
                          <strong>{progress.toFixed(progress >= 99.95 ? 0 : 1)}%</strong>
                          <small>
                            {formatBytes(torrent.downloaded)} of {formatBytes(torrent.size)}
                          </small>
                        </div>
                      </td>

                      <td data-label="Status">
                        <span className={`dl-state state-${state.kind}`}>{state.label}</span>
                      </td>

                      <td data-label="Seeds / Peers">
                        <div className="dl-swarm">
                          <strong>↑ {torrent.num_seeds}</strong>
                          <span>↓ {torrent.num_leechs}</span>
                        </div>
                      </td>

                      <td data-label="Speed">
                        <div className="dl-speed-stack">
                          {torrent.dlspeed > 0 && <span className="speed-down">↓ {formatSpeed(torrent.dlspeed)}</span>}
                          {torrent.upspeed > 0 && <span className="speed-up">↑ {formatSpeed(torrent.upspeed)}</span>}
                          {torrent.dlspeed <= 0 && torrent.upspeed <= 0 && <span>—</span>}
                        </div>
                      </td>

                      <td data-label="ETA">{formatEta(torrent.eta)}</td>
                      <td data-label="Size">{formatBytes(torrent.size)}</td>
                      <td data-label="Ratio">{formatRatio(torrent.ratio)}</td>

                      <td data-label="Category / Tags">
                        {editingOrganization===torrent.hash?<div className="dl-org-editor"><select value={editCategory} onChange={event=>setEditCategory(event.target.value)}><option value="">Uncategorized</option>{categories.map(item=><option value={item.name} key={item.name}>{item.name}</option>)}</select><input value={editTags} onChange={event=>setEditTags(event.target.value)} placeholder="tag1, tag2"/><div><button type="button" disabled={busy} onClick={()=>void saveTorrentOrganization(torrent)}>{busy?'Saving…':'Save'}</button><button type="button" disabled={busy} onClick={()=>setEditingOrganization(null)}>Cancel</button></div></div>:<div className="dl-meta-stack dl-org-display"><strong>{torrent.category || 'Uncategorized'}</strong><span>{torrent.tags || 'No tags'}</span><button type="button" onClick={()=>beginOrganizationEdit(torrent)}>Edit</button></div>}
                      </td>

                      <td data-label="Tracker">
                        <div className="dl-tracker">
                          <strong>{torrentSource(torrent).primary}</strong>
                          <span>{torrentSource(torrent).secondary}</span>
                        </div>
                      </td>

                      <td data-label="Availability">{formatAvailability(torrent.availability)}</td>
                      <td data-label="Added">{relativeTime(torrent.added_on)}</td>

                      <td className="dl-row-actions" data-label="Actions">
                        <button
                          type="button"
                          disabled={busy}
                          onClick={() => void toggleTorrent(torrent)}
                          title={state.active ? 'Stop' : 'Start'}
                        >
                          {state.active ? 'Ⅱ' : '▶'}
                        </button>
                        {torrent.oberiz_job_id != null && (
                          <button
                            type="button"
                            disabled={busy}
                            className="danger"
                            onClick={() => {
                              setDeleteFiles(false)
                              setRejectMode(true)
                              setDeleteTarget(torrent)
                            }}
                            title="Reject this release: remove it and never pick it again"
                            aria-label="Reject release"
                          >
                            ⊘
                          </button>
                        )}
                        <button
                          type="button"
                          disabled={busy}
                          className="danger"
                          onClick={() => {
                            setDeleteFiles(false)
                            setRejectMode(false)
                            setDeleteTarget(torrent)
                          }}
                          title="Delete"
                        >
                          ×
                        </button>
                      </td>
                    </tr>
                  )
                })}
              </tbody>
            </table>
          </div>
        )}
      </section>

      {rejectedReleases.length > 0 && (
        <details className="dl-card dl-rejected">
          <summary>Rejected releases ({rejectedReleases.length})</summary>
          <p>Oberiz will not pick these again for the title shown. Undo one to let it be chosen again.</p>
          <ul>
            {rejectedReleases.map((item) => (
              <li key={item.id}>
                <div>
                  <strong>{item.release_title}</strong>
                  <small>
                    {item.media_title ?? `Removed ${item.media_type}`}
                    {item.indexer_name ? ` · ${item.indexer_name}` : ''}
                  </small>
                </div>
                <button type="button" className="dl-secondary" onClick={() => void undoRejection(item.id)}>
                  Undo
                </button>
              </li>
            ))}
          </ul>
        </details>
      )}

      <section className="dl-bottom-grid">
        <article className="dl-card dl-health">
          <div className="dl-card-heading">
            <div>
              <span className="dl-card-icon">♡</span>
              <strong>Queue Health</strong>
            </div>
            <small>{lastUpdated ? `Updated ${lastUpdated.toLocaleTimeString()}` : 'Waiting…'}</small>
          </div>

          <div className="dl-health-main">
            <strong>{queueHealth}%</strong>
            <span className={queueHealth >= 90 ? 'healthy' : 'attention'}>
              <i />
              {queueHealth >= 90 ? 'Everything looks good' : 'Queue needs attention'}
            </span>
          </div>

          <div className="dl-health-grid">
            <div><strong>↓ {totals.downloading}</strong><span>Downloading</span></div>
            <div><strong>↑ {totals.seeding}</strong><span>Seeding</span></div>
            <div><strong>≡ {totals.queued}</strong><span>Queued</span></div>
            <div><strong>! {totals.stalled}</strong><span>Stalled</span></div>
          </div>
        </article>

        <article className="dl-card dl-bandwidth">
          <div className="dl-card-heading">
            <div>
              <span className="dl-card-icon">▥</span>
              <strong>Bandwidth Usage</strong>
            </div>
            <small>Last 90 seconds</small>
          </div>

          <div className="dl-bandwidth-values">
            <span className="speed-down">↓ {formatSpeed(totals.downloadSpeed)}</span>
            <span className="speed-up">↑ {formatSpeed(totals.uploadSpeed)}</span>
          </div>

          <div className="dl-chart">
            <div className="dl-chart-grid" />
            <Sparkline points={speedHistory} field="down" />
            <Sparkline points={speedHistory} field="up" />
          </div>
        </article>

        <article className="dl-card dl-recent">
          <div className="dl-card-heading">
            <div>
              <span className="dl-card-icon">✓</span>
              <strong>Recent Completed</strong>
            </div>
            <small>{recentCompleted.length} shown</small>
          </div>

          {recentCompleted.length === 0 ? (
            <div className="dl-recent-empty">No completed torrents yet.</div>
          ) : (
            <div className="dl-recent-list">
              {recentCompleted.map((torrent) => (
                <div key={torrent.hash}>
                  <div className="dl-recent-thumb">{torrent.name.slice(0, 1).toUpperCase()}</div>
                  <div>
                    <strong>{torrent.name}</strong>
                    <span>{formatBytes(torrent.size)} · {relativeTime(torrent.completion_on)}</span>
                  </div>
                  <b>✓</b>
                </div>
              ))}
            </div>
          )}
        </article>
      </section>

      {magnetOpen && (
        <div className="dl-modal-backdrop" onMouseDown={() => setMagnetOpen(false)}>
          <form className="dl-modal" onSubmit={submitMagnet} onMouseDown={(event) => event.stopPropagation()}>
            <div className="dl-modal-heading">
              <div>
                <p className="dl-kicker">NEW DOWNLOAD</p>
                <h2>Add magnet</h2>
              </div>
              <button type="button" onClick={() => setMagnetOpen(false)}>×</button>
            </div>

            <label>
              Magnet URL
              <textarea
                autoFocus
                required
                value={magnet}
                onChange={(event) => setMagnet(event.target.value)}
                placeholder="magnet:?xt=urn:btih:..."
              />
            </label>

            <label>
              Category <span>(optional)</span>
              <input
                value={category}
                onChange={(event) => setCategory(event.target.value)}
                placeholder="movies"
              />
            </label>

            <div className="dl-modal-actions">
              <button type="button" className="dl-secondary" onClick={() => setMagnetOpen(false)}>
                Cancel
              </button>
              <button type="submit" className="dl-primary">
                Add to qBittorrent
              </button>
            </div>
          </form>
        </div>
      )}

      {deleteTarget && (
        <div className="dl-modal-backdrop" onMouseDown={() => setDeleteTarget(null)}>
          <div className="dl-modal dl-delete-modal" onMouseDown={(event) => event.stopPropagation()}>
            <div className="dl-modal-heading">
              <div>
                <p className="dl-kicker danger-kicker">{rejectMode ? 'REJECT RELEASE' : 'DELETE TORRENT'}</p>
                <h2>{rejectMode ? 'Reject this release?' : 'Remove download?'}</h2>
              </div>
              <button type="button" onClick={() => setDeleteTarget(null)}>×</button>
            </div>

            <p className="dl-delete-name">{deleteTarget.name}</p>
            {rejectMode && (
              <p className="dl-delete-hint">
                The torrent is removed from qBittorrent and Oberiz will never pick this release again for this
                title; its next search will look for another one. You can undo it from the manual search.
              </p>
            )}

            <label className="dl-check">
              <input
                type="checkbox"
                checked={deleteFiles}
                onChange={(event) => setDeleteFiles(event.target.checked)}
              />
              <span>
                <strong>Also delete downloaded files</strong>
                <small>This permanently removes the data from disk.</small>
              </span>
            </label>

            <div className="dl-modal-actions">
              <button type="button" className="dl-secondary" onClick={() => setDeleteTarget(null)}>
                Cancel
              </button>
              <button type="button" className="dl-danger-button" onClick={() => void confirmDelete()}>
                {rejectMode
                  ? deleteFiles ? 'Reject + delete files' : 'Reject release'
                  : deleteFiles ? 'Delete torrent + files' : 'Delete torrent'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

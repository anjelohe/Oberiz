type Props = {
  name: string
  size?: number
}

const paths: Record<string, string> = {
  menu: 'M4 7h16M4 12h16M4 17h16',
  dashboard: 'M3 11.5 12 4l9 7.5v8a1.5 1.5 0 0 1-1.5 1.5h-5v-6h-5v6h-5A1.5 1.5 0 0 1 3 19.5z',
  movies: 'M4 4h16v16H4zM8 4v16M16 4v16M4 8h4M4 16h4M16 8h4M16 16h4',
  series: 'M5 7h14v12H5zM8 3l4 4 4-4',
  profiles: 'M4 7h10M18 7h2M14 4v6M4 17h2M10 17h10M6 14v6M4 12h6M14 12h6M10 9v6',
  downloads: 'M12 3v12m0 0-4-4m4 4 4-4M4 19h16',
  imports: 'M4 5h16v14H4zM8 9h8M8 13h5M15 16l2 2 3-4',
  indexers: 'M5 6c0-1.1 3.1-2 7-2s7 .9 7 2-3.1 2-7 2-7-.9-7-2Zm0 6c0 1.1 3.1 2 7 2s7-.9 7-2M5 18c0 1.1 3.1 2 7 2s7-.9 7-2M5 6v12M19 6v12',
  history: 'M3 12a9 9 0 1 0 3-6.7L3 8m0-5v5h5M12 7v5l3 2',
  calendar: 'M6 3v3m12-3v3M4 9h16M5 5h14a1 1 0 0 1 1 1v14H4V6a1 1 0 0 1 1-1Zm3 8h3m2 0h3m-8 4h3',
  settings: 'M12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6Zm8 3 2-1-2-4-2 .5a8 8 0 0 0-2-2L16.5 3h-5L11 5.5a8 8 0 0 0-2 2L7 7 5 11l2 1a8 8 0 0 0 0 2l-2 1 2 4 2-.5a8 8 0 0 0 2 2l.5 2.5h5l.5-2.5a8 8 0 0 0 2-2l2 .5 2-4-2-1a8 8 0 0 0 0-2Z',
  search: 'm21 21-4.4-4.4M19 11a8 8 0 1 1-16 0 8 8 0 0 1 16 0Z',
  plus: 'M12 5v14M5 12h14',
  plusCircle: 'M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0ZM12 8v8M8 12h8',
  check: 'm5 12 4 4L19 6',
  trash: 'M4 7h16M9 7V4h6v3m-8 0 1 13h8l1-13M10 11v5M14 11v5',
  bell: 'M18 9a6 6 0 0 0-12 0c0 7-3 7-3 9h18c0-2-3-2-3-9M10 21h4',
  database: 'M4 5c0-1.1 3.6-2 8-2s8 .9 8 2-3.6 2-8 2-8-.9-8-2Zm0 0v7c0 1.1 3.6 2 8 2s8-.9 8-2V5m-16 7v7c0 1.1 3.6 2 8 2s8-.9 8-2v-7',
  downloadbox: 'M12 3v11m0 0 4-4m-4 4-4-4M4 17v3h16v-3',
  chevronDown: 'm6 9 6 6 6-6',
  chevronUp: 'm6 15 6-6 6 6',
  play: 'm8 5 11 7-11 7z',
  more: 'M6 12h.01M12 12h.01M18 12h.01',
}

export function Icon({ name, size = 22 }: Props) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d={paths[name] ?? paths.dashboard} />
    </svg>
  )
}

// Header-only variants: match Cinetta without changing icons elsewhere in Oberiz.
export function SidebarIcon({name,size=24}:Props){
  return <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
    {name==='dashboard'?<><path d="M3 10.5 12 3l9 7.5"/><path d="M5.5 9.5V20h13V9.5"/></>
      :name==='movies'?<><rect x="3" y="4.5" width="18" height="15" rx="2.5"/><path d="M3 9h18M8 4.5v4.5M16 4.5v4.5"/></>
      :name==='series'?<><rect x="4" y="5" width="16" height="11" rx="2.5"/><path d="M10 20h4M12 16v4"/></>
      :<path d={paths[name]??paths.dashboard}/>}
  </svg>
}

export function HeaderIcon({ name, size = 22 }: Props) {
  return <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={name==='search'||name==='bell'||name==='menu'?2:1.8} strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
    {name==='search'?<><circle cx="11" cy="11" r="7"/><path d="m20 20-3.8-3.8"/></>
      :name==='bell'?<><path d="M18 8a6 6 0 0 0-12 0c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.7 21a2 2 0 0 1-3.4 0"/></>
      :name==='settings'?<><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></>
      :<path d={paths[name]??paths.search}/>}
  </svg>
}

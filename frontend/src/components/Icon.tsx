type Props = {
  name: string
  size?: number
}

const paths: Record<string, string> = {
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

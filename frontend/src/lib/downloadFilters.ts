export function matchesDownloadMetadata(
  torrent: { category: string; tags: string },
  tracker: string,
  filters: { tracker: string; category: string; tag: string },
): boolean {
  return (!filters.tracker || tracker === filters.tracker)
    && (!filters.category || JSON.stringify(torrent.category) === filters.category)
    && (!filters.tag || torrent.tags.split(',').map(tag => tag.trim()).includes(filters.tag))
}

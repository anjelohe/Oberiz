# Libraries across multiple disks — development on October 14, 2026

Historical description of the workflow released in v1.0.0. The workflow after v1.0.1 is described in [Simplified storage](Oberiz_Almacenamiento_Simplificado.md): read-only local scanning or Cinetta inventory through the API, without Oberiz-managed SMB connections.

## Configuration

Settings → Library folders has independent movie and series lists. Each entry supports typing or browsing for a folder, adding another, and removing it. “Load category folders” queries qBittorrent; selecting a category copies its path into the list using the configured remote mappings. It does not modify the category or its qBittorrent destination.

Folders must be accessible from the Oberiz server, not necessarily from the browser. Docker requires mounts; remote qBittorrent requires remote-to-local path mappings. New import destinations remain separate, single-value fields. Changing recognition folders does not move, rename, or delete content.

Legacy Movies/Series paths remain initial roots until a list is saved. An explicitly empty list does not restore the old path. Duplicate entries are removed and overlapping trees are traversed only once.

## Scanning and downloads

- Configurable manual and periodic scanning: initially 120 minutes, disabled until enabled and saved.
- Configured roots are scanned before a title's first automatic search. RSS and automation also check equivalent qBittorrent torrents.
- Completed external torrents are recognized through accessible content paths, even when other episodes of that series are already indexed. Queries are reused; paths are checked periodically or when metadata changes.
- A pending or paused torrent prevents obtaining another copy of the same episode/movie; it is not resumed automatically. An inaccessible completed torrent still prevents duplicates without pretending that its file is indexed.
- If a disk is disconnected, cannot be read, or appears empty after previously containing recorded files, previous availability is preserved. Automatic acquisition for the affected media type stops until the path recovers and a scan succeeds.
- Removing a configured folder does not delete files or associations. Known associations are preserved during rescanning.
- Upgrades remain subject to Allow upgrades, cutoff, and installed quality. Monitoring is not disabled when a download completes.

## Series and statistics

Each season distinguishes missing, partially available, up to date, and complete. The latest season of an ongoing series is not considered complete merely because every known episode is present. Unknown status is treated conservatively.

“Follow upcoming episodes” is a per-season control, independent of following future seasons. It stores a start date: episodes airing afterward remain monitored after their air date passes. Later manual exclusions are respected. It does not recover episodes preceding that date.

The Dashboard adds monitored series and available, aired missing, and future episodes in a two-column card. Cards adapt to available space. Indexed storage totals recognized files, not disk capacity or every torrent; configuration and scanning remain exclusively in Settings.

The folder browser and category list support multiple selection. Windows network prefixes such as `\\server\share` are preserved correctly; drafts such as `UNC\server\share` produced by the older browser are repaired on save. Entirely empty mappings are ignored rather than preventing saving.

## Integrations

The public API returns `availability_status` and `seasons` with per-season counts and states. To follow upcoming episodes in an already partially available season, use `requested_seasons: [2]`, `monitor_mode: "future"`, and `monitor_future_seasons: false`. External clients should use this information to avoid disabling an ongoing season because some episodes are available. The external portal UI is not part of this repository and was not changed here.

Added internal route: `GET /api/library/roots`. Settings accepts `movie_library_paths`, `series_library_paths`, `qbittorrent_path_mappings`, `library_scan_enabled`, and `library_scan_interval_minutes`. Root states include accessibility, last scan, and error.

## Verification

Tests use temporary databases and folders: multiple disks, overlapping roots, disconnection, empty mounts, external episode detection with another copy already indexed, preserved torrent states, and persistent future monitoring. Migration preserves existing installation data. Frontend compilation, six-language coverage, and backend static analysis were also verified.

A pre-change ZIP backup was saved privately on the maintainer's desktop, including code, configuration, Git history, and consistent SQLite snapshots. It contains private data and must not be published.

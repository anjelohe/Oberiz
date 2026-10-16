# Library, renaming, and recovery — October 15, 2026

Included in v1.0.2. Publishing does not automatically update the installed instance or modify the NAS/existing torrents: install the new version explicitly.

## Responsibilities

- qBittorrent downloads to its category destination. Oberiz no longer imports, copies, creates hardlinks, moves files, or builds reseeding folders.
- Cinetta sends inventory to `PUT /api/v1/library`, with the public API and key enabled. Selecting Cinetta in Oberiz does not activate Cinetta's sender.
- Alternatively, choose locally accessible folders for read-only scans. There is no managed SMB configuration, credentials, or download path translation. OS-managed mounts are external to Oberiz; their permissions are unchanged.
- Inventory source and renaming are independent settings.

## Configuration

1. Configure categories and destinations in qBittorrent.
2. In **Library inventory**, choose **Cinetta** or **Local folders**.
3. For Cinetta, enable public API v1, generate a key, and save. Configure the Oberiz URL/key and enable inventory in Cinetta settings. The last accepted report date is displayed. Acquisitions continue even when Cinetta does not report. The optional pause checkbox, disabled by default including on upgrade, holds new acquisitions if the report is absent or older than 24 hours. The last inventory is retained without scanning remote paths.
4. For local folders, add one or more, save, and run or schedule a scan. Scanning does not move, rename, or delete files.
5. In **qBittorrent renaming and recovery**, renaming is **disabled by default**, including upgrades from older installations. Activation requires the new `qbittorrent_rename_enabled` field; legacy `rename_enabled` sent by older clients does not activate remote operations. It affects download jobs still awaiting processing, not the entire library or already-completed jobs retroactively.

## Optional renaming

Only completed downloads associated with Oberiz jobs are processed. The file list is retrieved through `GET /api/v2/torrents/files`; a complete plan is calculated and saved before any changes. The extension and current directory are preserved. `setLocation` is not used and a torrent is not split across disks.

The operation uses `POST /api/v2/torrents/renameFile`, not the visible torrent name. Each step is verified by fetching files again; HTTP 200 alone is insufficient. Retries recognize steps already applied before a disconnection and do not repeat renames or delete the journal.

Movie torrents containing several videos, unidentifiable episodes, external subtitles, invalid paths, or duplicate destinations require manual review. Such plans are not partially applied. If a valid plan fails after some steps, the journal and confirmed changes are preserved; no blind file rollback is attempted.

Besides checking torrent inventory, Oberiz queries the destination directory through qBittorrent `app/getDirectoryContent` before the first change and again before every step. A failed or unsupported query blocks renaming. SMB is unnecessary: qBittorrent reads the names. An occupied destination blocks the change. During recovery, relinking to an existing destination is allowed only when the source file does not also exist.

Real testing found that qBittorrent 5.2.3 `renameFile` can overwrite an unrelated file without an HTTP conflict, so its internal torrent list or an assumed 409 is insufficient. The added guard was verified with a real decoy file using Oberiz Rust code.

The API has no atomic “rename without replacement” operation. Do not share a destination among torrents managing the same files or run other organizers on those paths concurrently. External changes between checking and renaming, and hidden entries not exposed by the API, are limitations of the design without filesystem access. Universal detection is not promised.

## Archiving and recovery

By default, the original `.torrent`, hash, qBittorrent destination, file indices/sizes, and name mappings are retained in SQLite `torrent_archives`. Normal database backups include them; video backups or SMB access are unnecessary. Private tracker keys inside a `.torrent` are also secrets: protect backups.

Renaming requires this archive even if optional retention is disabled. It does not replace a complete qBittorrent session/configuration backup: statistics, queues, and all per-torrent preferences are not retained. Before resuming recovery, review file selection/priorities, categories, and qBittorrent limits. If export or validation fails, renaming does not begin and the job is not marked complete for automatic cleanup.

**Downloads and recovery → Recover stopped** requires confirmation and:

- refuses to modify a torrent already present in qBittorrent;
- verifies `.torrent` identity (v1/hybrid recovery; pure v2 currently requires manual recovery);
- adds it with `paused=true` and `stopped=true`, automatic management disabled, checking not skipped, and the original layout preserved;
- applies recorded names through qBittorrent and requests a recheck;
- never calls start/resume: users review and resume in qBittorrent;
- does not apply sharing cleanup to stopped recovery. It is reconsidered only after observing that the user resumed seeding.

Old import records and external `.torrent` files are preserved but are not automatically converted into remote recoveries: reliable mappings between legacy local and client destinations are missing. The UI no longer offers legacy reseeding that copied files; recover these cases manually in qBittorrent. Records with a remote archive are not deleted by obsolete-record cleanup.

## Safe upgrade

Migration 0022 preserves history, indexed files, folders, and legacy settings needed for compatibility/rollback. Startup first creates a consistent SQLite backup at `data/backups/oberiz-before-storage-simplification-*.sqlite3` if an older database exists. If backup fails, the transition does not proceed. Only legacy SMB credentials configuration is subsequently removed; OS mounts are not disconnected and the NAS is untouched.

Policies remain a separate feature. Legacy “remove torrent and original” policies become “remove torrent, keep files.” Manual deletion in Downloads allows file deletion through an unchecked checkbox and a confirmation with a persistent warning. qBittorrent executes that command even without Oberiz NAS access. Automatic policies and release rejection still preserve files.

Legacy fields and the mapping-suggestions endpoint remain temporarily for compatibility. The endpoint returns an empty list and does not query disks or qBittorrent. New download processing neither configures nor uses path translation.

## Verification and limitations

Tests use test SQLite, synthetic files, simulated clients, and loopback HTTP. No NAS or user torrents were accessed. The existing Cinetta JavaScript sender was reviewed and its tests passed; this does not modify Cinetta or confirm that sending is enabled in the installed user instance.

Real **qBittorrent v5.2.3** was also tested with an isolated profile, loopback-only WebUI, ephemeral credentials, and a private 8 KiB trackerless torrent: renaming, export preserving the hash, removal preserving data, stopped recovery, 100% recheck, and collision rejection through the real Rust guard.

Reproduce with `node scripts/test-qbittorrent-isolated.mjs <absolute-qbittorrent-path>`.
Private evidence: `backend/backups/storage-simplification-20261015/real-qbt-9RfBU6/result.json`.
The isolated process was closed afterward. NAS installation integration and Cinetta inventory activation remain user configuration tasks.

References: [official qBittorrent API](https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-5.0)), [official controller implementation](https://github.com/qbittorrent/qBittorrent/blob/master/src/webui/api/torrentscontroller.cpp).

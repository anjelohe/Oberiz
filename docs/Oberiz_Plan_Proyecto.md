# Oberiz — Product plan

## v1.0.7 publication — October 19, 2026

Release authorized: configurable daily cycles and persistent randomized plans.
Update versions, release notes and memory, verify backend/frontend, then publish
the new tag and generate packages. Remote deployment remains user-managed.

## Local configurable daily automation

Replace the interval input with daily cycle count (1–24), use saved randomized
daily windows, and expose a default-on restart-persistence checkbox. Preserve
legacy interval API compatibility. Test calendar boundaries, persistence,
single-claim slots, settings changes and no catch-up bursts. Manual/on-add
searches are additional. Local only; prohibited trackers must remain disabled.

## Local restart-safe automation scheduling

Persist next automatic execution across service restarts and power outages.
Reserve deadlines before work and preserve manual execution. Test database reopen,
repeated restarts, missed intervals, and interval changes. Local only; no release
requested. This does not make automation permitted on trackers that prohibit it.

## v1.0.6 — remote-test release

Publication authorized October 19, 2026. Package selected-season monitoring and
cached Cinetta episode binding for user-managed remote testing. Keep the separate
overnight scheduler incident unconfirmed; no remote deployment is authorized.

## Cached series inventory binding — October 19, 2026

Reuse previously received Cinetta episode identities when series metadata creates
managed rows and during acquisition preflight. Verify repeated binding is
idempotent and does not update inventory freshness. The automation interval is
a search interval, not a Cinetta filesystem rescan. Keep remote scheduler incident
diagnosis separate from this reproduced availability defect. Local only.

## Local requested-season fix — October 19, 2026

Preserve already-aired episodes when accepting explicit seasons plus future
seasons from Cinetta. Verify partial availability (one of six present), missing
episode selection, refresh persistence, newly discovered episodes, and manual
exclusions. Existing installations need the updated backend and affected requests
reapplied; no live database edits or publication are performed here.

## v1.0.5 search rebuild — October 17, 2026

User authorized republishing the existing v1.0.5 with the search fix and rebuilding
packages. Keep public notes English and leave installed services untouched.

## Local search follow-up

Combine original and punctuation-normalized indexer searches even with initial
hits, deduplicate, then apply unchanged profiles. Confirmed against the XBytes
colon case and tested with local HTTP fixtures. Pending backend restart in the
development instance. Publication approval was subsequently granted above.

## v1.0.5 publication — October 17, 2026

User authorized publication of caching, configurable intervals and incremental
receiver support. Cinetta sender deployment is separate. Release packages are
built by GitHub Actions; local installation and qBittorrent remain untouched.

## Local October 17 follow-up

Implemented configurable cache and incremental inventory in Oberiz/Cinetta.
See [design, intervals and limits](Oberiz_Incremental_Inventory.md). Pending real
installation testing and explicit publication approval; no deployment performed.

## v1.0.4 rebuild: reuse received inventory

Missing reports use the last Cinetta inventory already submitted through the
existing Oberiz API/key. Remove the second connection from Settings. Migration
0024 retains unmanaged identities independently, with additive/offline safeguards.
Refresh recalculates, not synchronizes Cinetta. Await a normal inventory report
after upgrade before first use. The user authorized republishing as v1.0.4 and
rebuilding all packages; no installed-service deployment is requested.

## v1.0.4 — code uploaded, October 16, 2026

Main and tag `v1.0.4` were pushed to GitHub at commit `ca0d1d3`.
CI `38929148251` and installer workflow `38929148331` were initially recorded as running. The v1.0.4 GitHub release subsequently became available with five assets; deployment to the local service was not performed.

Missing content reports and the interface redesign were visually approved. Security and qBittorrent session-reuse changes passed local verification: 124 unit tests (one ignored), two migration tests, frontend checks, and Clippy after fixing DNS/login, metadata memory, report-cache retention, and episode ranges. The user authorized publication. Real qBittorrent memory consumption still needs observation. Initial password setup over the network remains supported. See project memory and audit.

## Published v1.0.3 foundation

Remote initial setup, confirmed manual deletion with optional files, an opt-in Cinetta pause disabled by default, and dynamic version display.

Movie/series libraries, automation/RSS, profiles, qBittorrent, indexers, API, and six languages. Cinetta inventory or read-only local scanning, without direct imports or Oberiz-managed SMB connections. Optional remote renaming disabled by default, persistent journal, and stopped recovery. See [project memory](Oberiz_Memoria_Actualizada.md) and [storage guide](Oberiz_Almacenamiento_Simplificado.md).

## Principles

1. Reliable automation and decisions based on files that actually exist.
2. No torrent deletion, pausing, or cancellation through implicit external state changes.
3. Protect configuration, credentials, data, and libraries during upgrades.
4. Separate inventory from operations: receiving paths does not grant file access.
5. Keep credentials out of Git and API responses.
6. Public repository documentation, descriptions, and release notes are written in English. Preserve application localization and protocol identifiers.

## Next operational checks

- Install while preserving data and verify the pre-migration-0022 backup.
- Configure inventory sending in the installed Cinetta instance or read-only local folders.
- Verify remote qBittorrent and directory queries before enabling renaming; avoid concurrent organizers on the same destinations.
- The isolated qBittorrent 5.2.3 test already verifies renaming, export/hash, stopped recovery, recheck, and occupied-destination protection. It does not replace checks for each installation.
- Recovery of old import records and pure v2 torrents remains manual.
- Select content and limits before real acquisition tests.
- Validate Linux/Docker/Windows packages in real installations and maintain recovery/migration tests.
- Observe real qBittorrent memory consumption after the connection changes.

Publishing code does not configure credentials or change NAS permissions automatically. Changing a product version does not justify deleting user data.

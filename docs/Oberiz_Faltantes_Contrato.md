# Missing content reports — agreed contract, October 16, 2026

Producer reference: sibling Cinetta project,
`docs/cinetta-oberiz-missing-contract-2026-10-16.md`.

Status: included in v1.0.4. Publishing does not deploy it to production.
Testing with real Cinetta credentials and libraries is still pending.

## Access

- Cinetta provides `GET /api/v1/library/inventory` with a read-only Bearer token, snapshots, cursors, and filters. A query does not access the NAS, scan, or mutate data.
- Oberiz provides GET `/api/v1/library/missing` and POST `/api/v1/library/missing/refresh`; GET never starts jobs.
- External integration requires X-Api-Key and explicit `api.missing_reports_enabled=true`, disabled by default even on upgrade. Without permission, return 403 before querying providers or creating jobs. Existing inventory PUT and request authorization are unchanged.
- The Oberiz panel uses an administrator session and does not require external permission.
- POST has durable idempotency; queries use a stable opaque job_id. Pagination must retain all filters and job_id, never switch to another job's latest report.
- GET without a previous report returns 409 report_not_found (not 404, reserved for an unsupported route or nonexistent job_id). Cinetta presents this as “Refresh to create a report.”

## Result semantics

Only aired episodes and released movies, never future titles or those without a confirmed date. TMDB metadata uses es-ES, standard aired numbering, and season 0 for specials. TVDB IDs are resolved through /find only when there is exactly one match, without title matching. Valid air_date and release_date values must be on or before today UTC. There is no country filter or guarantee of digital availability; a theatrically released movie may appear.

Reports are stored separately: no title creation, monitoring, requests, downloads, renaming, or media deletion.

The current Cinetta inventory always declares complete:false. Missing candidates are provisional: state=partial, warning inventory_incomplete, and availability=unknown, even when the result is zero. Do not claim “nothing is missing.” Offline or unknown indexed content counts as present to prevent false missing results.

A multi-episode file (S01E01E02) may declare optional `episode_number_end`, at most 50 above `episode_number`; the whole range counts as present. Without it, only `episode_number` counts and later episodes in the range appear missing. Arithmetic is overflow-safe and inventory expansion is limited to 500,000 unique episodes.

While a job is queued or running, GET without job_id returns the latest finished report for that filter. The empty job placeholder is returned only if no finished report exists. Reports are a cache: when full, the oldest finished reports are evicted rather than permanently rejecting checks. The latest valid report for the scope being refreshed is protected; invalid connection configuration does not trigger global eviction.

A correct comparison must also handle metadata failures, incompatible numbering, and unknown identifiers without title-based matching.

## Large series and pagination

Maximum 100 fragments per page, 500 items per fragment, and 1 MiB per response. Always obey the most restrictive limit. Split series by season, then by fragment; repeat identifiers and include season/part in the title. Sort collections by date and ID and split similarly. Order and pages are immutable for each job_id, without omitted or duplicated items.

Report totals count distinct logical groups and unique episodes/movies, not fragments as separate series. An oversized individual item produces partial and report_item_too_large, never silent truncation.

## Access and local testing

Movies > Incomplete collections; Series > Missing episodes; an individual action on each card. Select a series/collection by ID or the entire inventory. Opening the dialog makes no query. Latest report sends GET; Refresh sends POST with Idempotency-Key. Stop waiting cancels only browser waiting, not the server job.

Settings > Missing content reports: Cinetta base URL, a library:read key created in Cinetta, and optional external permission. Save and configure TMDB. The internal panel does not require enabling the public API or external permission; the Cinetta button does.

The Cinetta key is stored in the private DB like other connection credentials and is never returned to the browser. Protect the DB/backups with ACLs; no new full-database encryption is added. Internal routes are /api/library/missing and /refresh; lookup /api/library/missing/collection?movie_tmdb_id=ID resolves collections without creating titles.

Jobs and idempotency use missing_reports and missing_report_actions (migration 0023). At most two jobs are active; matching active scopes are merged. Restart marks interrupted jobs as errors without restarting queries and preserves the latest nonfailed report. Timeouts: 15 seconds per request and 20 minutes per job. No redirects or TLS verification bypass. Only the report persists, not the full Cinetta inventory.

Limits: 32 MiB inventory, 500,000 entries, 1,000 series, 200 seasons per series, 2,000 movies for collection discovery, and 20,000 report items. Title/season/item limits produce partial with a warning. Per-title/season metadata errors preserve partial results. Metadata responses are consumed incrementally rather than preloaded. ETag is not used yet. An expired cursor or changed snapshot fails without mixing pages.

Start an isolated copy with `./scripts/start-missing-preview.ps1 -Build` (PowerShell 7). Open http://127.0.0.1:2033. Separate data/config lives in backend/backups/missing-preview, ignored by Git. It does not touch the existing service. Create a test password and configure Cinetta/TMDB. Titles do not need to be imported into Oberiz to query inventory. Stop only the PID returned by the script.

Automated tests use simulated local inventory/TMDB HTTP and in-memory SQLite: future/unknown dates, specials, offline content, large series, idempotency, permissions, and absence of library/request/download mutations. Frontend tests cover explicit actions, fixed pagination, and errors preserving the previous report. These do not replace testing with real libraries or certify DVD/absolute numbering.

TMDB references: [seasons](https://developer.themoviedb.org/reference/tv-season-details), [collections](https://developer.themoviedb.org/reference/collection-details), and [external IDs](https://developer.themoviedb.org/reference/find-by-id).

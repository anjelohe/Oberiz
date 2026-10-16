# Missing content reports — agreed contract, October 16, 2026

Producer reference: sibling Cinetta project,
`docs/cinetta-oberiz-missing-contract-2026-10-16.md`.

Status: the shared-inventory simplification is included in the authorized v1.0.4 rebuild. Publishing does not deploy the installed service.
Testing with real Cinetta credentials and libraries is still pending.

## Access

- Cinetta sends inventory through the existing `PUT /api/v1/library` using the existing Oberiz API key. Oberiz does not call a Cinetta inventory endpoint and requires no Cinetta URL or second key. The accepted inventory is cached independently of managed titles.
- Oberiz provides GET `/api/v1/library/missing` and POST `/api/v1/library/missing/refresh`; GET never starts jobs.
- External integration requires X-Api-Key and explicit `api.missing_reports_enabled=true`, disabled by default even on upgrade. Without permission, return 403 before querying providers or creating jobs. Existing inventory PUT and request authorization are unchanged.
- The Oberiz panel uses an administrator session and does not require external permission.
- POST has durable idempotency; queries use a stable opaque job_id. Pagination must retain all filters and job_id, never switch to another job's latest report.
- GET without a previous report returns 409 report_not_found (not 404, reserved for an unsupported route or nonexistent job_id). Cinetta presents this as “Refresh to create a report.”

## Result semantics

Only aired episodes and released movies, never future titles or those without a confirmed date. TMDB metadata uses es-ES, standard aired numbering, and season 0 for specials. TVDB IDs are resolved through /find only when there is exactly one match, without title matching. Valid air_date and release_date values must be on or before today UTC. There is no country filter or guarantee of digital availability; a theatrically released movie may appear.

Reports are stored separately: no title creation, monitoring, requests, downloads, renaming, or media deletion.

The received inventory does not certify physical completeness, so reports always treat it as incomplete. Missing candidates are provisional: state=partial, warning inventory_incomplete, and availability=unknown, even when the result is zero. Do not claim “nothing is missing.” Offline or unknown indexed content counts as present to prevent false missing results.

A multi-episode file (S01E01E02) may declare optional `episode_end` in the incoming PUT (normalized internally to `episode_number_end`), at most 50 above `episode_number`; the whole range counts as present. Without it, only `episode_number` counts and later episodes in the range appear missing. Arithmetic is overflow-safe and inventory expansion is limited to 500,000 unique episodes.

While a job is queued or running, GET without job_id returns the latest finished report for that filter. The empty job placeholder is returned only if no finished report exists. Reports are a cache: when full, the oldest finished reports are evicted rather than permanently rejecting checks. The latest valid report for the scope being refreshed is protected; invalid connection configuration does not trigger global eviction.

A correct comparison must also handle metadata failures, incompatible numbering, and unknown identifiers without title-based matching.

## Large series and pagination

Maximum 100 fragments per page, 500 items per fragment, and 1 MiB per response. Always obey the most restrictive limit. Split series by season, then by fragment; repeat identifiers and include season/part in the title. Sort collections by date and ID and split similarly. Order and pages are immutable for each job_id, without omitted or duplicated items.

Report totals count distinct logical groups and unique episodes/movies, not fragments as separate series. An oversized individual item produces partial and report_item_too_large, never silent truncation.

## Access and local testing

Movies > Incomplete collections; Series > Missing episodes; an individual action on each card. Select a series/collection by ID or the entire inventory. Opening the dialog makes no query. Latest report sends GET; Refresh sends POST with Idempotency-Key. Stop waiting cancels only browser waiting, not the server job.

Settings > Missing content reports explains reuse of received inventory and offers optional external permission only. Configure TMDB normally. The internal panel does not require enabling external report permission; the Cinetta button does. Refresh recalculates against the latest received inventory and never requests a new Cinetta synchronization.

No second Cinetta key is stored. Migration 0024 removes the obsolete URL/key settings. Protect the private inventory cache and backups with ACLs; no new full-database encryption is added. Internal routes are /api/library/missing and /refresh; lookup /api/library/missing/collection?movie_tmdb_id=ID resolves collections without creating titles.

Jobs and idempotency use missing_reports and missing_report_actions (migration 0023). At most two jobs are active; matching active scopes are merged. Restart marks interrupted jobs as errors without restarting queries and preserves the latest nonfailed report. Timeouts: 15 seconds per request and 20 minutes per job. No redirects or TLS verification bypass. The normalized received inventory also persists in received_library_inventory (migration 0024), atomically with acceptance of the original PUT. It includes titles not managed by Oberiz without creating movie/series/request/download records. Additive omissions and offline/empty roots preserve previous presence, matching receiver semantics. Reidentification replaces entries for that path. Only explicitly confirmed omissions under available nonempty roots remove entries.

Limits: 32 MiB inventory, 500,000 entries, 1,000 series, 200 seasons per series, 2,000 movies for collection discovery, and 20,000 report items. Title/season/item limits produce partial with a warning. Per-title/season metadata errors preserve partial results. Metadata responses are consumed incrementally rather than preloaded. The received cache is limited to 32 MiB and 500,000 entries; exceeding it rejects the PUT and rolls back its changes rather than partially saving inventory. Each report captures one immutable cache version when admitted.

Start an isolated copy with `./scripts/start-missing-preview.ps1 -Build` (PowerShell 7). Open http://127.0.0.1:2033. Separate data/config lives in backend/backups/missing-preview, ignored by Git. It does not touch the existing service. Create a test password, configure TMDB, and send an inventory PUT with the existing Oberiz API key. Titles do not need to be imported into Oberiz to check inventory. After upgrading, the first report requires a fresh normal Cinetta inventory submission; old managed-title data cannot reconstruct previously ignored titles. Until then refresh returns 409 cinetta_inventory_not_received. inventory_generated_at records Oberiz receipt time, not a new provider scan time. Stop only the PID returned by the script.

Automated tests use simulated local inventory/TMDB HTTP and in-memory SQLite: future/unknown dates, specials, offline content, large series, idempotency, permissions, and absence of library/request/download mutations. Frontend tests cover explicit actions, fixed pagination, and errors preserving the previous report. These do not replace testing with real libraries or certify DVD/absolute numbering.

TMDB references: [seasons](https://developer.themoviedb.org/reference/tv-season-details), [collections](https://developer.themoviedb.org/reference/collection-details), and [external IDs](https://developer.themoviedb.org/reference/find-by-id).

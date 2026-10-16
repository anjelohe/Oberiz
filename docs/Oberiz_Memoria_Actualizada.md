# Oberiz — Project memory

## English public documentation — October 16, 2026

The user requests English throughout the public GitHub repository. Translate documentation, repository metadata, and release notes without changing application localization, protocol identifiers, release tags, or installers. Retain historical filenames to preserve links. The v1.0.4 release notes were already translated in commit `5bcef71`; this pass covers the remaining documentation and older releases.

Repository description and all five release bodies were updated in English,
preserving each release's five assets and existing changelog links. Twenty Markdown
documents were checked for local links and balanced code fences. No issues/PRs
were returned by the repository issue listing; the wiki remote was not available.
The package-list API returned 403 because the credential lacks read:packages;
package-page descriptions were therefore not verified or changed. No code,
application language, upstream indexer definition, old commit, or release tag is
rewritten by this documentation pass.

## Confirmed delivery — October 16, 2026

- Code and documentation uploaded to `anjelohe/Oberiz`, branch `main`, commit `ca0d1d3` (`Release v1.0.4: missing reports, interface refresh and security hardening`).
- Annotated tag `v1.0.4` pushed atomically with main.
- GitHub confirmed CI and installer-build startup. The original delivery check recorded both as running: [CI](https://github.com/anjelohe/Oberiz/actions/runs/38929148251) and [installers](https://github.com/anjelohe/Oberiz/actions/runs/38929148331). The release was later confirmed available with five assets when its notes were translated.
- Local verification: 124 unit tests passed, one ignored; two migration tests; five frontend suites, frontend build, formatting, and Clippy. OSV checked 252 active versions without advisories; npm reported no vulnerabilities.
- Includes subsequent indexer-catalog changes and the final review's four corrections, in addition to missing content reports, visual changes, and security hardening documented below.
- No local installation update or qBittorrent restart. Measuring real memory consumption is pending: tests do not establish that this symptom has been resolved. Initial password setup over the network is unchanged.
- Post-delivery annotations were initially local and were not part of the tagged commit. Later documentation-only commits do not move or overwrite v1.0.4.

## Authorized follow-up closure — v1.0.4, October 16, 2026

The user authorized correcting four findings and publishing. Fixed: DNS pinning for authenticated aliases and manual login validation; incremental metadata consumption; retention of the current scope's latest valid report during eviction; overflow-safe episode ranges with bounded expansion. Blocking notes below are historical. Initial password setup over the network is unchanged. No service deployment or qBittorrent restart; real memory behavior remains to be observed.

Final verification: 124 passing unit tests (one ignored), two migration tests, five frontend suites and compilation, formatting, and Clippy. OSV: 252 active versions without advisories; npm: zero vulnerabilities. Main and v1.0.4 publication authorized; installers are built by GitHub Actions, not the local machine.

## Post-security review — October 16, 2026 (historical)

New changes were reviewed without modifying their code. Publication was held for four regressions: DNS pinning/login-alias validation, metadata preloading without a cumulative bound, eviction of the current scope's last valid report, and episode-range overflow. See the audit findings. At that stage, 120 unit tests, frontend, formatting, and Clippy passed but did not cover these cases. No commit, tag, or push had yet been created. This state was superseded by the closure above.

## Security and qBittorrent — publication authorized October 16, 2026

After stopping publication and completing the audit, the user authorized v1.0.4 publication. Requested fixes: reused qBittorrent HTTP sessions, serialized renewal and hash-filtered queries, and non-overlapping UI polling. Torrent contents and qBittorrent preferences are unchanged. Stronger IP/redirect filtering; memory/file/report/login-attempt quotas; absolute Windows-tool paths; sanitized errors. Restore preserves current security identity and connections, revoking sessions as before. UI confirmation updated in six languages.

INITIAL PASSWORD SETUP remains available OVER THE NETWORK, as explicitly requested. Push code and v1.0.4 to rebuild on GitHub; do not update the installed service or restart qBittorrent. See [audit and local closure](Oberiz_Auditoria_Seguridad_2026-10-16.md). Real memory consumption remains unmeasured, and old sessions may persist until expiration. Do not restart qBittorrent automatically.

## Version 1.0.4 — October 16, 2026

The user approved publishing the locally tested missing reports and visual changes. Versions are synchronized across frontend, backend, OpenAPI, Compose, and workflow. See [release notes](releases/v1.0.4.md).

Approved design: borderless cards in all themes, SVG navigation, Middle as the default only for new installations, custom favicon, aligned header, and content 20 px higher than the previous design. Dashboard shows three downloads up to 1920 px and four above, aligned columns, uniform buttons, and a white circled plus in Add content. Publication does not require restarting the local service or modifying private data. Attachments, backups, and credentials stay out of Git.

## Missing reports — included in 1.0.4, October 16, 2026

Engine, jobs, and endpoints are implemented separately from library, requests, and downloads. Cinetta library:read inventory and TMDB metadata, with dates on or before today UTC. Entry points in Movies, Series, and individual titles; connection and opt-in external permission in Settings. See [contract and local test](Oberiz_Faltantes_Contrato.md). Included in 1.0.4; the local installation is not deployed automatically.

## Agreed adjustments — October 15, 2026 · v1.0.3

Initial password configurable remotely without requiring loopback; configure on a trusted network. Manual torrent deletion has an unchecked file-deletion checkbox and persistent warning; automatic actions preserve data. Pausing for absent/stale Cinetta inventory is optional and disabled by default, including upgrades. Sidebar uses package.json version rather than a stale literal. These adjustments supersede the v1.0.2 restrictions below.

## Architecture change — October 15, 2026 · v1.0.2

The architecture is documented in [Simplified storage](Oberiz_Almacenamiento_Simplificado.md). Removed direct importing, copying/hardlinks/moving, and SMB connections. Selectable inventory: Cinetta API or read-only local scans, without automatic switching after disconnection. Last-report date and a hold on new acquisitions without Cinetta inventory less than 24 hours old were introduced in this version.

Optional renaming exclusively through qBittorrent, disabled by default with new consent independent of legacy import settings. SQLite stores the `.torrent`, hash, destination, and journal; recovery starts stopped with recheck and no automatic start. Legacy history preserved; legacy recovery remains manual. No changes to the NAS, real torrents, active installation, or Cinetta repository.

The sender exists in `cinetta/src/services/oberizInventory*.js`; the earlier Rust-only review was incomplete. Its 11 synthetic tests passed.

Real qBittorrent 5.2.3 with an isolated profile passed renaming, export/hash, stopped recovery, and recheck. Testing discovered that its API can overwrite unrelated files; a remote directory check through qBittorrent was added and tested, blocking occupied destinations and failing safely when unavailable. There is no atomic guarantee against other writers.

The following sections describe the earlier published state, not the new workflow.

Historical product version at this point: **1.0.3**. The description below starts with the first **1.0.0** release; October 15 changes are summarized above. Publication does not automatically update existing installations.

## Purpose and architecture

Self-hosted movie/series automation server: TMDB/TVDB metadata, Cardigann indexers, quality/language profiles, search, RSS, qBittorrent, importing, real library, calendar, history, and request API.

Rust/Axum/SQLite backend and React/TypeScript frontend. Windows distribution (portable and service installer), Linux, and Docker. SQLite migrations are retained to protect existing data; resetting repository history does not reset user databases or libraries.

## Included features in the historical foundation

- Movies, series, seasons, and episodes with monitoring and inherited profiles.
- Multi-disk library: independent folders, multiple selection, qBittorrent categories, and path mappings.
- Configurable pre-search, manual, and periodic scanning. Existing files are unchanged; availability survives inaccessible disks and duplicate downloads are avoided.
- Incomplete/up-to-date/complete season states; persistent upcoming-episode tracking independent of future seasons. API availability and counts.
- Responsive Dashboard with movies, series/episodes, downloads, indexers, and queue. Middle uses orange controls and blue monitoring indicators.
- Settings organized into nine rows: appearance/support/diagnostics; RSS/qBittorrent/sharing policies; TMDB/TVDB/automation; imports/naming; paths; library; maintenance/API; backups; password.
- Combined download filters for tracker, category, tag, state, and text.
- Web YAML import: validation, 512 KiB limit, safe writes, confirmed replacement, and preserved credentials. New definitions are not automatically enabled.
- Cardigann `:contains` applies text conditions; invalid selectors are not broadened by stripping restrictions. Numeric CSS escapes and relative `:has` combined with `:contains` are not interpreted.
- Consistent scoring for discovered/imported movies, with cutoff and improvement margin respected by RSS.
- Hardlink/copy/move import, renaming, transfer records, reseeding, and seeding policies. Do not move a file still being seeded without considering torrent consequences.
- Six languages: Spanish, English, French, German, Italian, and European Portuguese.

## Historical SMB and permissions

Optional native Windows SMB connections: machine DPAPI credentials, picker shares, UNC paths, and restoration at application/service startup. Independent of NAS brand and Cinetta. No drive-letter assignment, forced disconnections, or SMB1.

Without SMB configuration, existing access, permissions, and paths remain; the picker still opens disks/shares. The service does not inherit drives connected in another Windows session. NAS accounts retain their actual permissions: no read-only account is required or automatically configured. Linux/Docker uses OS SMB mounts.

Saved shares remain visible offline; opening them may fail. An offline NAS does not block startup indefinitely. Moving machines requires reentering credentials. The API does not return passwords or encrypted blobs. See [SMB](Oberiz_SMB.md).

## Integrations

Public API v1 with API keys, idempotency, profiles, selected seasons, and monitoring. Cinetta must consume real availability and distinguish upcoming episodes from future seasons. Its UI changes belong to its repository, not the Oberiz package. Optional Overseerr compatibility.

qBittorrent categories and states are preserved during scanning. Mappings remain necessary in this historical workflow when reported paths differ from those accessible to Oberiz.

## Historical verification and limitations

87 unit tests and one migration test passed; optional live-storage test excluded from CI. Clippy, formatting, frontend build, filters, and language coverage verified.

The RSS → simulated download → import → rescan → upgrade cycle uses temporary SQLite, local HTTP RSS, and a simulated client, including failure/recovery cases. DPAPI encryption, corrupt-data rejection, secret-free responses, listing without NAS access, and saved-credential removal were checked.

An optional real-code move probe between two SMB shares verified content and removed temporary files without changing existing content. It ran interactively and does not prove LocalSystem permissions. NAS authentication from the service requires credential configuration and operational verification. No real content downloads were part of automated tests.

Build dependencies remain locked in Cargo.lock/package-lock.json. Dependabot configuration is retained with version-update PRs paused; protections and functional dependencies are not removed.

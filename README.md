# Oberiz

Self-hosted media automation for movies and series. Oberiz brings metadata, indexers, quality profiles, searches, qBittorrent, importing and the real library together in one application.

**Current version: 1.0.0 — continuous RSS-based automation.**

## What it does

Oberiz covers the complete workflow:

```text
Add media from TMDB
→ apply quality and language profile
→ search Cardigann indexers
→ score and select a release
→ send to qBittorrent with verified category, tags and hash
→ download and import
→ rename, rescan, upgrade and record history
```

For series, it manages seasons, episodes, monitoring and inherited profiles. Automation prefers a complete-series release when appropriate, then a season pack, then individual episodes. Active duplicate grabs are prevented through job state and magnet infohash tracking.

## Included features

- Movies and series with TMDB metadata.
- A real library with indexed files, availability, rescans and score-based upgrades.
- Quality Profiles and Language Profiles, with default profiles per media type.
- Cardigann indexers, an upstream catalogue and local private definitions.
- qBittorrent integration: connection checks, real categories, profile tags, progress and torrent controls.
- Scheduled automation, importing, hardlinks/copy/move, reseeding and seed policies.
- Per-indexer RSS: incremental sync, persistent deduplication, Wanted matching, scoring and traceable automatic grabs.
- Calendar, History, Dashboard with real data and a Public API v1 for Cinetta.
- Optional Radarr/Sonarr compatibility for clients such as Overseerr, isolated from the native API.
- Local diagnostics without secrets and automated release-selection tests.

## Architecture

| Component | Technology | Location |
| --- | --- | --- |
| Backend | Rust, Axum, Tokio, SQLx and SQLite | `backend/` |
| Frontend | React, TypeScript and Vite | `frontend/` |
| Indexers | Cardigann definitions | `config/indexers/` |
| Documentation | Product state, API, roadmap and history | `docs/` |

The backend listens on `http://localhost:2032`. The development frontend listens on `http://localhost:5173` and proxies `/api` to the backend.

## Run locally

In one terminal:

```powershell
cd backend
cargo run
```

In another:

```powershell
cd frontend
npm install
npm run dev
```

Then open `http://localhost:5173`.

## Initial configuration

1. Open **Settings** and add a TMDB credential.
2. Configure qBittorrent and select **Test Connection**.
3. Define movie, series and download paths.
4. Sync or add indexers, then configure quality profiles.
5. Add a movie or series and run Automation once the configuration is verified.

Private definitions belong in `config/indexers/custom/`. SQLite databases, secrets, private definitions and build artifacts are excluded from version control.

## API and diagnostics

The internal API is served below `/api`. Public API v1 is enabled in Settings and uses the `X-Api-Key` header:

```text
GET  /api/v1/status
GET  /api/v1/quality-profiles
GET  /api/v1/requests
POST /api/v1/requests
GET  /api/v1/requests/{id}
```

Cinetta uses the native API v1: it reads authenticated profiles and links every request with an idempotent client identifier. Each profile explicitly declares whether it accepts standard or 4K requests, so Cinetta only offers destinations matching the requested quality. Overseerr compatibility is enabled separately and exposes `/radarr/api/v3/*` and `/sonarr/api/v3/*`; it does not replace or constrain the native integration.

`GET /api/diagnostics` reports the version, operating system, SQLite status, service configuration, indexers, automation and recent errors without exposing keys or passwords. RSS exposes `GET /api/rss/status` and can be run manually with `POST /api/rss/run`.

From **Settings → Backup & Restore**, you can create and restore consistent SQLite snapshots. They are stored in `backend/backups` when Oberiz starts from `backend` and include pending WAL changes. Restoring reverts Oberiz data, but never modifies media files or qBittorrent torrents.

## Verification

```powershell
cd backend
cargo test

cd ..\frontend
npm run build
```

The test suite covers release parsing/scoring and the `Complete Series → Season Pack → Episode` decision. The manual end-to-end procedure is documented in `docs/Oberiz_Memoria_Actualizada.md`.

## Documentation

Product documentation is stored in [`docs/README.md`](docs/README.md): technical state, API integration, importing, roadmap, RSS and version history. The repository root is intentionally limited to this README, the license, configuration and source code.

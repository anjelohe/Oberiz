# Oberiz

Self-hosted media automation for movies and series. Oberiz brings metadata, indexers, quality profiles, searches, qBittorrent, importing and the real library together in one application.

**Current version: 1.0.0 — initial public release with secure admin access, media automation and ready-to-run packages.**

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

Oberiz is served on `http://localhost:2032`. The web interface and API are delivered by the same service.

## Prebuilt downloads

Every release provides ready-to-run packages. No Rust, Node.js or frontend build tools are required on the target machine.

| Platform | Download | Starts with |
| --- | --- | --- |
| Docker | `ghcr.io/anjelohe/oberiz:v1.0.0` | `docker compose up -d` |
| Linux x86_64 | `oberiz-1.0.0-linux-x86_64.tar.gz` | `sudo ./install.sh` |
| Windows x86_64 (portable) | `oberiz-1.0.0-windows-x86_64.zip` | `Start-Oberiz.bat` |
| Windows x86_64 (installer) | `oberiz-1.0.0-windows-x86_64-setup.exe` | Setup assistant |

The packages are available from the [v1.0.0 release](https://github.com/anjelohe/Oberiz/releases/tag/v1.0.0). The release workflow also builds and attaches the Windows package whenever a version is published.

## Support Oberiz

Oberiz is free to use. If you would like to support its development, donations are optional and appreciated through [PayPal](https://paypal.me/anjelohe).

<p align="center"><a href="https://paypal.me/anjelohe"><img src="frontend/public/paypal-donate-qr.png" alt="Donate to Oberiz through PayPal" width="180"></a></p>

TMDB is configured independently by each user with their own API credential and subject to TMDB’s terms.

## Docker installation

Install Docker Engine and the Docker Compose plugin, then run:

```bash
git clone https://github.com/anjelohe/Oberiz.git
cd Oberiz
docker compose up -d
```

Open `http://localhost:2032`.

`oberiz-data` stores the database and backups, while `oberiz-config` stores indexer definitions. Both survive container updates. Before starting, set `OBERIZ_MEDIA_PATH` in your shell or `.env` if Oberiz needs to import media files; it is mounted inside the container at `/media`. The published image is downloaded automatically; no local build tools are needed.

The published Docker image is public and does not require a GitHub account. If GitHub ever returns a permissions error while pulling it, authenticate with `docker login ghcr.io -u anjelohe` and use a personal access token with the `read:packages` permission.

## LXC installation

Docker is the recommended way to run Oberiz in an LXC. Create a 64-bit Debian or Ubuntu LXC with enough disk space for the application data and media mounts, enable Docker support for the container (in Proxmox, enable **Nesting**), then follow the Docker installation above.

Open Oberiz from another device with `http://LXC_IP:2032`. If the LXC is unprivileged, make sure its mapped user can read and write any host folders mounted for media and downloads.

For an LXC without Docker, use the Linux package instead. It requires a distribution with systemd; system containers without systemd should use Docker.

## Linux installation

The Linux package targets 64-bit Linux distributions using systemd. Download `oberiz-1.0.0-linux-x86_64.tar.gz` from the release, extract it and run:

```bash
tar -xzf oberiz-1.0.0-linux-x86_64.tar.gz
cd oberiz-1.0.0-linux-x86_64
sudo ./install.sh
```

It installs Oberiz in `/opt/oberiz`, creates a persistent data directory at `/var/lib/oberiz`, and starts the `oberiz` system service. Open `http://localhost:2032`. To remove the application while retaining its data, run `sudo ./uninstall.sh` from the extracted package.

The service uses the `oberiz` system account. Grant that account the required read/write permissions for the media, download and qBittorrent paths you configure in Settings.

## Windows portable installation

Download `oberiz-1.0.0-windows-x86_64.zip` from the release, extract it anywhere you want to keep Oberiz, then double-click `Start-Oberiz.bat`. It starts the included `Oberiz.exe` invisibly in the background and opens `http://127.0.0.1:2032`; no command window remains open.

The `data` and `config` folders are created beside the executable, so the installation is portable and can be moved or backed up as one directory.

## Windows installer

Download `oberiz-1.0.0-windows-x86_64-setup.exe` from the release and run it. The setup assistant installs Oberiz, creates Start Menu and optional desktop shortcuts, and offers to start it at the end. Oberiz then runs invisibly in the background, without a command window.

Application files are installed under `Program Files\Oberiz`. Your database, configuration and backups are stored separately in `%LOCALAPPDATA%\Oberiz`, so they survive application updates and uninstallation.

To stop Oberiz, close `Oberiz.exe` from **Task Manager**. Starting it again from the shortcut opens the web interface.

## Network security

Oberiz uses an administrator password and revocable browser sessions. Login attempts are rate-limited after repeated failures. For access from other devices, place Oberiz behind an HTTPS reverse proxy and set `OBERIZ_COOKIE_SECURE=true` in the service or container environment. This marks the session cookie as HTTPS-only. Leave it unset when using `http://localhost:2032` directly.

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

### Configure Overseerr

1. In **Oberiz → Settings → Public API v1**, generate or enter an API key and enable **Overseerr compatibility**.
2. In **Overseerr → Settings → Services**, add Oberiz as a Radarr service for movies using `http://your-oberiz-host:2032/radarr` and the Oberiz API key.
3. Add it again as a Sonarr service for series using `http://your-oberiz-host:2032/sonarr` and the same key.

Oberiz provides the configured quality profiles, language profiles and movie/series root folders to Overseerr. The compatibility layer is intended for Overseerr requests; Oberiz remains the source of truth for searches, categories, tags and importing.

`GET /api/diagnostics` reports the version, operating system, SQLite status, service configuration, indexers, automation and recent errors without exposing keys or passwords. RSS exposes `GET /api/rss/status` and can be run manually with `POST /api/rss/run`.

From **Settings → Backup & Restore**, you can create and restore consistent SQLite snapshots. They are stored in the persistent Oberiz data directory and include pending WAL changes. Restoring reverts Oberiz data, but never modifies media files or qBittorrent torrents.

## Documentation

Product documentation is stored in [`docs/README.md`](docs/README.md): technical state, API integration, importing, roadmap, RSS and version history.

# Oberiz — API and integration

## Current scope

Oberiz exposes an internal API for its interface and a **Public API v1** for external clients. API v1 registers and retrieves requests; it does not yet aim to provide complete compatibility with Radarr, Sonarr, Overseerr, or Seerr.

## Activation and authentication

Enable **Public API v1** and generate a key in Settings. All calls require `X-Api-Key`:

```http
X-Api-Key: obrz_your_key
```

The current implementation does not use `Authorization: Bearer`.

## Current public endpoints

| Method | Path | Description |
| --- | --- | --- |
| `GET` | `/api/v1/status` | Oberiz status and version. |
| `GET` | `/api/v1/quality-profiles` | Active quality profiles for native clients, with `request_quality` (`standard` or `4k`). |
| `GET` | `/api/v1/requests` | Latest 250 requests. |
| `POST` | `/api/v1/requests` | Create a request or reapply an expanded season selection from a native client. |
| `GET` | `/api/v1/requests/{id}` | Request details and computed status. |

## Creating a request

For Cinetta, `client_name: "cinetta"` and `client_request_id` provide a stable request identity. A normal retry returns the existing request. Resubmitting a series with `requested_seasons` updates that selection in Oberiz, allowing later seasons to be added without creating a duplicate series. This operation processes only that media item, not a global automation cycle.

Local Cinetta states are not destructive commands to Oberiz. Changing a card to pending does not implicitly delete or cancel torrents. Any future remote pause or cancellation must be explicit.

```http
POST /api/v1/requests
X-Api-Key: obrz_your_key
Content-Type: application/json
```

Minimal movie request:

```json
{
  "media_type": "movie",
  "tmdb_id": 19995
}
```

Series with a profile and monitoring mode:

```json
{
  "media_type": "series",
  "tmdb_id": 1399,
  "quality_profile_id": 4,
  "client_name": "cinetta",
  "client_request_id": "184",
  "monitored": true,
  "monitor_mode": "all",
  "requested_by": "user_42"
}
```

Supported fields:

- `media_type`: `movie` or `series`.
- `tmdb_id`: required TMDB identifier.
- `quality_profile_id`: optional; the default profile is used when omitted.
- `client_name` and `client_request_id`: identify the request in the client. Together they make creation idempotent: a retry returns the same Oberiz request.
- `monitored`: optional; defaults to `true`.
- `monitor_mode`: optional for series; defaults to `all`.
- `requested_seasons`: optional for series; the exact list of season numbers to monitor.
- `monitor_future_seasons`: optional for series; tracks future episodes without enabling unselected past seasons.
- `requested_by`: optional audit text.

After creation, Oberiz updates metadata and schedules an automation cycle in the background.

## Request states

Current states are computed from the library and download jobs:

```text
movie:  available | downloading | searching
series: available | downloading | searching | monitoring
```

A request without linked media is shown as `pending`. These states do not yet include progress, quality, or speed metrics. Clients requiring them must use the internal Oberiz API or wait for an API v1 extension.

## Download routing

External clients do not select paths, categories, or tags. Oberiz resolves the effective Quality Profile and applies:

- qBittorrent category;
- tag template;
- language profile and acceptance rules;
- upgrade and cutoff policy.

For native clients, each profile also has `request_quality`: `standard` or `4k`. Configure it using **4K profile for API requests** in Profiles. Cinetta first filters by media type and this flag: if one profile remains it submits it directly; if several remain, users can choose only among those profiles.

The category and tags are verified after adding the torrent. `qb_hash` is the main correlation between Oberiz and qBittorrent.

## Cinetta and compatibility

Cinetta uses only this native API. It retrieves authenticated profiles, retains the remote identifier, and lets Oberiz determine paths, categories, tags, and import handling.

Radarr/Sonarr compatibility is independent and enabled only through `overseerr.compat_enabled`. Its `/radarr/api/v3/*` and `/sonarr/api/v3/*` routes support clients such as Overseerr; they do not determine the native API.

For Overseerr, configure the same instance twice:

```text
http://host:2032/radarr  → Movies
http://host:2032/sonarr  → Series
```

The layer exposes profiles, a root folder, languages, tags, lookup, and movie/series creation. Sonarr resolves a series TVDB identifier to TMDB within Oberiz. Compatibility has been validated for the Overseerr request workflow but does not replicate the full Radarr/Sonarr API.

A shared key can be used during initial setup. For an exposed installation or multiple clients, regenerate it after testing; a future improvement is revocable per-integration keys.

## Local diagnostics

`GET /api/diagnostics` is an internal, read-only support endpoint. It does not require the public API key and does not return secrets. It includes version, system, SQLite status, service configuration, indexers, automation, and recent errors.

## Local backup

The Settings interface uses these internal routes; they are not exposed through the public API:

- `GET /api/backups`: list available snapshots.
- `POST /api/backups`: create a self-contained, WAL-consistent SQLite snapshot.
- `GET /api/backups/{filename}/download`: download a local backup.
- `POST /api/backups/upload`: upload a SQLite file as a restorable backup.
- `DELETE /api/backups/{filename}`: delete a local backup.
- `POST /api/backups/{filename}/restore`: restore a compatible snapshot after explicit UI confirmation.

Backups are stored in the `backups` folder under the directory from which Oberiz starts. The optional scheduler creates backups every 1–720 hours and retains the configured number. Restoration replaces Oberiz data, including settings and history, and does not touch media libraries or external torrents. Current security settings and connections are preserved by the v1.0.4 restore policy.

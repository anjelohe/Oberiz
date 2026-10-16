# Oberiz — Importing, renaming, and seeding policies

Historical v1.0.0/v1.0.1 document. Direct transfers and SMB were removed in v1.0.2. Optional renaming is performed through qBittorrent and starts disabled. See [Simplified storage](Oberiz_Almacenamiento_Simplificado.md).

## Operational note — October 14, 2026

The `move` method was verified using the real code between two NAS shares with a temporary file, without changing existing content. This verifies the connection used by the test, not LocalSystem permissions. `auto` remains a different method: it tries a hardlink and falls back to copying; it does not mean moving.

The optional [SMB connections](Oberiz_SMB.md), included in v1.0.0, provided UNC access from the service. Without configuring them, previous paths and access remained unchanged. Moving requires source read/delete and destination write access; browsing or scanning does not prove these permissions. The warning about files being seeded by qBittorrent remains: an SMB connection does not eliminate that risk.

## 1. Problem

A private tracker may require X days of seeding, ratio X, or both.

Directly moving/renaming a file being seeded by qBittorrent can break the torrent.

## 2. Recommended solution

### Hardlink

If Downloads and Library use the same filesystem:

```text
Downloads:
Avatar.2009.EXTENDED.1080p.BluRay.x265-RARB.mkv

Library:
Avatar (2009)/Avatar (2009) - 1080p BluRay x265.mkv
```

Both names refer to the same physical data.

Benefits:

- qBittorrent keeps seeding;
- clean library names;
- no duplicated storage;
- deleting the original after meeting seeding requirements does not remove the library copy.

### Copy fallback

Across different filesystems:

- copy;
- retain the original until seeding requirements are met.

## 3. Rule

```text
if seeding_required:
    hardlink_or_copy
else:
    move_or_hardlink
```

## 4. Per-indexer seed policy

```json
{
  "min_seed_time_minutes": 10080,
  "min_ratio": 1.0,
  "requirement_mode": "both",
  "cleanup_mode": "remove_torrent_and_original"
}
```

Modes:

- time;
- ratio;
- either;
- both;
- manual.

Cleanup:

- remove_torrent_keep_files;
- remove_torrent_and_original;
- manual;
- never.

## 5. Private trackers

If an indexer is private and has no policy:

```text
NO AUTO DELETE
```

## 6. Available qBittorrent data

Oberiz already receives:

- ratio;
- seeding_time;
- completion_on;
- uploaded;
- downloaded;
- state;
- category;
- tags;
- tracker/source.

Still to persist at the time of this design:

```text
download hash ↔ release ↔ media ↔ indexer
```

## 7. Naming templates

Movies:

```text
{Title} ({Year}) - {Resolution} {Source} {Codec}
```

Series:

```text
{SeriesTitle} - S{Season:00}E{Episode:00} - {EpisodeTitle} - {Resolution} {Source} {Codec}
```

## 8. Import cycle

```text
qBittorrent Completed
→ DownloadJob Completed
→ identify files
→ ignore samples/extras
→ determine Movie/Episode
→ calculate final name
→ hardlink/copy
→ record MediaFile
→ Available
→ keep seeding
→ meet SeedPolicy
→ safe cleanup
```

---

## Per-profile qBittorrent routing

The qBittorrent category belongs to the Quality Profile because it represents that content type's destination/workflow.

Examples (category identifiers are illustrative and retained unchanged):

```text
Movie 1080p -> peliculasWD19
Movie 4K    -> peliculasWD19_4k
Series 1080p -> seriesWD19
Series 4K    -> seriesWD19_4k
```

Use a tag for the tracker/indexer:

```text
qBittorrent Tags: [tracker]
```

Tags are resolved during Grab and saved in `DownloadJob` so later reseeding preserves the original routing.

Oberiz retains one reserved technical tag, `_oberiz_job_<id>`, for qBittorrent correlation.

---

## Episode-aware series since 0.7.0

The importer recognizes `SxxExx` and multi-episode ranges.

When it identifies an episode, it:

- records `media_files`;
- sets `series_episodes.has_file = true`;
- creates an `episode_files` relationship.

Recommended series template:

```text
{Title} - S{Season:00}E{Episode:00} - {EpisodeTitle}
```

Multi-file packs are imported file by file when filenames contain recognizable numbering.

The qBittorrent category still determines where qBittorrent downloads. The final library in this historical workflow uses `paths.series`.

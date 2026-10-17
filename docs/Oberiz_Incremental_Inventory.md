# Incremental inventory and metadata cache

Included in the authorized v1.0.5 release, October 17, 2026. Publishing does not deploy installed services automatically.
Both Oberiz and the Cinetta sender must be updated to use incremental submissions.
There is no additional connection or API key.

## Configurable intervals

Settings → Missing content reports provides:

| Setting | Default | Accepted range |
| --- | --- | --- |
| Metadata refresh | 7 days | 1–365 days |
| Finished/canceled series refresh | 90 days | 7–365 days |
| Periodic inventory check | 1 hour | 1–168 hours |
| Full inventory reconciliation | 7 days | 1–365 days |

Metadata intervals are cache lifetimes, not background download schedules.
Expired data is refreshed when a report needs it. Newly received inventory is
used immediately; comparing current dates does not require refreshing metadata.
Collections remain subject to the normal interval because new movies may be
added even when every currently known movie is present. Ended/canceled series
and their season metadata use the longer interval. A local report still walks
its selected scope; valid metadata is reused without another provider request.

Changing the periodic sender interval is learned on the sender's next normal
contact. Scan events can send changes earlier. If stale-inventory pausing is
enabled, choose a check interval comfortably below its fixed 24-hour threshold.

## Persistent, bounded cache

Migration 0025 adds missing_metadata_cache and inventory_sync_state. Cache keys
include provider base URL, language and resource path. Compact data excludes
images, credits and other unused fields. Capacity: 20,000 entries and 128 MiB of
JSON, with least-recently-used eviction. Jobs share serialized cache misses to
avoid duplicate provider requests. Errors are not cached as successful responses.
On provider failure, a cached response no older than one year can be used with
the metadata_stale report warning; this does not extend its freshness timestamp.

Cinetta forwards existing collection membership and full release dates when its
stored metadata provides them, without fetching new metadata just to send it.
Date-less presentation records are not substituted. Oberiz seeds missing entries
and merges newly known members, without resetting TTL on repeated deliveries or
overwriting newer provider fields. TMDB remains the fallback for unknown/expired
metadata and series catalogs. Reports remain provisional where physical
completeness cannot be certified.

## Versioned incremental submissions

Authenticated GET /api/v1/status adds inventory_sync with protocol=1, the accepted
revision, check_hours and full_days. The existing API key is reused. PUT
/api/v1/library accepts optional sync={revision,base_revision,full} and collections.
Revisions are 64-character hexadecimal opaque fingerprints.

Cinetta saves one connection-specific checkpoint only after acknowledgment.
Changed paths resend all identities for that path, including multi-episode files.
Unchanged checks send only a lightweight heartbeat with roots, empty file arrays,
and revision information, keeping opt-in stale-inventory protection informed.
Unknown receiver revisions, missing checkpoints, periodic reconciliation or
omitted paths cause a full report. A mismatched delta returns 409
inventory_revision_mismatch without changing data. Replaying an accepted delta is
safe; restart/restore mismatches lead to full synchronization. Legacy receivers
continue receiving additive full reports.

Omissions never cause direct filesystem operations. Automatic removal confirmation
requires every omitted checkpoint path (at most 1,000) to return ENOENT on a
bounded verification; existing files, permissions, timeouts and I/O errors leave
the report additive. Receiver offline/empty-root protection still applies.
Disconnected or unreadable roots abort preparation. No NAS file is deleted by
this protocol. One sending installation is assumed; concurrent writers are not a
multi-client merge protocol.

The current sender still verifies inventory against filesystem state before
delivery. Incremental transport reduces HTTP payloads, not all physical file
checks. This favors conservative availability over assuming a DB omission is a
confirmed disk deletion.

## Verification

Tests cover cache hits, expiry, finished-series lifetimes, concurrent misses,
provider failure fallback, inventory changes without repeat provider requests,
settings validation, revision mismatch/replay, heartbeat preservation and collection
reuse. Cinetta tests cover changed-only payloads, reconciliation, legacy fallback,
rollback recovery, confirmed removals and multi-episode preservation. Tests use
synthetic data and local HTTP, not real NAS contents. No real-service deployment,
qBittorrent restart, new release or GitHub publication is performed by this change.

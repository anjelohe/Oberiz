# Cinetta inventory: security fixes

## Scope

Informational inventory only. Oberiz receives paths and metadata to determine availability, without instructing Cinetta to move, rename, or delete files. Select import.method=external in Storage; existing configuration is not changed automatically. Restart/rebuild to apply the corrected sources. This work does not deploy or touch the real database.

## Fixes

- API authorization before reading JSON. At most two simultaneous reports, without queued request bodies; 429 permits retrying. Body limited to 128 MiB, one million entries, and 1,024 roots; path, range, and size validation without overflow.
- External mode blocks reseeding even for old jobs, discovery, manual/scheduled scans, folder checks, and NAS suggestions. The UI does not offer reseeding in external mode or when settings cannot be checked.
- Imports, scans, and reseeding share a barrier with mode changes. Switching to external waits for active operations rather than interrupting them halfway. Destructive torrent deletion also uses the barrier; legacy-job cleanup in external mode does not request data deletion.
- Reports are additive by default. Omission does not establish absence. allow_large_removal=true explicitly confirms any omission, not just large removals, and requires an available root that has not suddenly become empty. Without roots, outside them, or when uncertain, availability is preserved.
- Reclassifying an episode as a movie clears links and recalculates episode availability. Simultaneously declaring incompatible identities for one path is rejected.
- External mode returns an empty superseded list and does not infer download ownership. In managed mode, linking requires a complete/mapped source path rather than a basename; partial replacement does not nominate multi-episode files.

## Validation and limitations

Integration with the other session's final snapshot: 111 unit tests and one migration test passed (112 total), with one live test skipped. Web build and four frontend checks passed. Strict validation of positive episodes and overflow-safe ranges from the other session is preserved. Already-published v1.0.0 installers/binaries are not changed automatically.

Tests use in-memory SQLite, a loopback HTTP server, and synthetic temporary files. Never run live_storage_move_probe during automated validation. No NAS, provider, real qBittorrent, or real library was used. Do not automatically enable cleanup or removal confirmation. One sending client with serialized submissions: no multi-client/snapshot-version protocol is implemented. Joint testing with Cinetta remains pending before enabling production submissions.

Backup and SHA-256 manifest in the Cinetta repository:
backups/oberiz-fixes-20261014-175230/baseline and MANIFEST.json.
Source transfer must first verify that the destination has not changed since the backup and then verify file hashes.

This document records the October 14 implementation; later storage behavior is documented in [Simplified storage](Oberiz_Almacenamiento_Simplificado.md).

CREATE TABLE missing_metadata_cache (
    cache_key TEXT PRIMARY KEY,
    data_json TEXT NOT NULL,
    fetched_at INTEGER NOT NULL,
    accessed_at INTEGER NOT NULL,
    final_series INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX missing_metadata_cache_lru ON missing_metadata_cache(accessed_at,cache_key);
CREATE TABLE inventory_sync_state (
    id INTEGER PRIMARY KEY CHECK(id=1),
    revision TEXT NOT NULL
);

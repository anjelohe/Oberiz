CREATE TABLE IF NOT EXISTS rss_sync_state (
    indexer_id TEXT PRIMARY KEY,
    last_synced_at TEXT,
    last_status TEXT NOT NULL DEFAULT 'idle',
    last_message TEXT,
    last_new_items INTEGER NOT NULL DEFAULT 0,
    last_matched_items INTEGER NOT NULL DEFAULT 0,
    last_grabbed_items INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS rss_processed_releases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    indexer_id TEXT NOT NULL,
    release_guid TEXT NOT NULL,
    title TEXT NOT NULL,
    download_url TEXT,
    published_at TEXT,
    outcome TEXT NOT NULL DEFAULT 'new',
    detail TEXT,
    processed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(indexer_id, release_guid)
);

CREATE INDEX IF NOT EXISTS idx_rss_processed_recent
ON rss_processed_releases(indexer_id, processed_at DESC);

INSERT INTO settings(key,value) VALUES ('rss.enabled','false') ON CONFLICT(key) DO NOTHING;
INSERT INTO settings(key,value) VALUES ('rss.interval_minutes','15') ON CONFLICT(key) DO NOTHING;

CREATE TABLE IF NOT EXISTS indexer_runtime (
    indexer_id TEXT PRIMARY KEY,
    last_status TEXT NOT NULL DEFAULT 'unknown'
        CHECK(last_status IN ('unknown','online','offline')),
    last_message TEXT,
    last_latency_ms INTEGER,
    last_checked_at TEXT,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_indexer_runtime_status
ON indexer_runtime(last_status, last_checked_at);

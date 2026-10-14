CREATE TABLE IF NOT EXISTS download_jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    media_type TEXT NOT NULL CHECK(media_type IN ('movie','series')),
    media_id INTEGER NOT NULL,
    release_title TEXT NOT NULL,
    indexer_id TEXT NOT NULL,
    indexer_name TEXT,
    category TEXT NOT NULL DEFAULT 'oberiz',
    qb_hash TEXT,
    status TEXT NOT NULL DEFAULT 'queued',
    source_path TEXT,
    library_path TEXT,
    import_method TEXT,
    file_mappings_json TEXT NOT NULL DEFAULT '[]',
    torrent_metadata_path TEXT,
    last_error TEXT,
    imported_at TEXT,
    cleaned_at TEXT,
    reseed_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_download_jobs_status
ON download_jobs(status, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_download_jobs_media
ON download_jobs(media_type, media_id, created_at DESC);

CREATE UNIQUE INDEX IF NOT EXISTS idx_download_jobs_qb_hash
ON download_jobs(qb_hash)
WHERE qb_hash IS NOT NULL AND qb_hash <> '';

CREATE TABLE IF NOT EXISTS media_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    media_type TEXT NOT NULL CHECK(media_type IN ('movie','series')),
    media_id INTEGER NOT NULL,
    download_job_id INTEGER,
    path TEXT NOT NULL UNIQUE,
    size_bytes INTEGER,
    quality_json TEXT NOT NULL DEFAULT '{}',
    source_release TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(download_job_id) REFERENCES download_jobs(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_media_files_media
ON media_files(media_type, media_id);

CREATE TABLE IF NOT EXISTS seed_policies (
    indexer_id TEXT PRIMARY KEY,
    min_seed_time_minutes INTEGER NOT NULL DEFAULT 0,
    min_ratio REAL NOT NULL DEFAULT 0,
    requirement_mode TEXT NOT NULL DEFAULT 'manual'
        CHECK(requirement_mode IN ('time','ratio','either','both','manual')),
    cleanup_mode TEXT NOT NULL DEFAULT 'remove_torrent_and_original'
        CHECK(cleanup_mode IN ('remove_torrent_keep_files','remove_torrent_and_original','manual','never')),
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO settings(key,value) VALUES
('import.enabled','true'),
('import.method','auto'),
('import.rename_enabled','true'),
('import.movie_template','{Title} ({Year}) - {Resolution} {Source} {Codec}'),
('import.series_template','{Title} - {Resolution} {Source} {Codec}'),
('import.keep_reseed_metadata','true'),
('import.cleanup_after_seed','true'),
('import.torrent_metadata_path','./data/torrents'),
('paths.reseed',''),
('ui.theme','dark')
ON CONFLICT(key) DO NOTHING;

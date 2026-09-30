ALTER TABLE media_files ADD COLUMN resolution TEXT;
ALTER TABLE media_files ADD COLUMN source TEXT;
ALTER TABLE media_files ADD COLUMN codec TEXT;
ALTER TABLE media_files ADD COLUMN hdr TEXT;
ALTER TABLE media_files ADD COLUMN audio TEXT;
ALTER TABLE media_files ADD COLUMN language TEXT;
ALTER TABLE media_files ADD COLUMN quality_score INTEGER NOT NULL DEFAULT 0;
ALTER TABLE media_files ADD COLUMN file_exists INTEGER NOT NULL DEFAULT 1;
ALTER TABLE media_files ADD COLUMN discovered_by TEXT NOT NULL DEFAULT 'import';
ALTER TABLE media_files ADD COLUMN verified_at TEXT;

CREATE INDEX IF NOT EXISTS idx_media_files_available
ON media_files(media_type, media_id, file_exists, quality_score DESC);

CREATE TABLE IF NOT EXISTS library_scans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    media_type TEXT NOT NULL,
    root_path TEXT NOT NULL,
    scanned_files INTEGER NOT NULL DEFAULT 0,
    matched_files INTEGER NOT NULL DEFAULT 0,
    unmatched_files INTEGER NOT NULL DEFAULT 0,
    started_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TEXT,
    error TEXT
);

CREATE TABLE IF NOT EXISTS media_requests (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    media_type TEXT NOT NULL CHECK(media_type IN ('movie','series')),
    tmdb_id INTEGER NOT NULL,
    media_id INTEGER,
    quality_profile_id INTEGER,
    monitored INTEGER NOT NULL DEFAULT 1,
    monitor_mode TEXT,
    requested_by TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_media_requests_created
ON media_requests(created_at DESC);

INSERT INTO settings(key,value) VALUES
('api.enabled','false'),
('api.key','')
ON CONFLICT(key) DO NOTHING;

UPDATE media_files SET
  resolution=COALESCE(resolution,json_extract(quality_json,'$.resolution')),
  source=COALESCE(source,json_extract(quality_json,'$.source')),
  codec=COALESCE(codec,json_extract(quality_json,'$.codec')),
  hdr=COALESCE(hdr,json_extract(quality_json,'$.hdr')),
  audio=COALESCE(audio,json_extract(quality_json,'$.audio')),
  language=COALESCE(language,json_extract(quality_json,'$.language')),
  quality_score=CASE WHEN quality_score=0 THEN COALESCE(CAST(json_extract(quality_json,'$.score') AS INTEGER),0) ELSE quality_score END,
  file_exists=1,
  discovered_by=CASE WHEN download_job_id IS NULL THEN 'scan' ELSE 'import' END
WHERE quality_json IS NOT NULL AND quality_json<>'';

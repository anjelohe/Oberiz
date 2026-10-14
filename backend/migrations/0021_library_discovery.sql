CREATE TABLE library_discovery_checks (
    media_type TEXT NOT NULL,
    media_id INTEGER NOT NULL,
    checked_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(media_type, media_id)
);
ALTER TABLE series ADD COLUMN production_status TEXT;
ALTER TABLE series ADD COLUMN monitor_future_since TEXT;
ALTER TABLE series_seasons ADD COLUMN follow_future_episodes INTEGER NOT NULL DEFAULT 0;
ALTER TABLE series_seasons ADD COLUMN follow_future_since TEXT;
INSERT INTO settings(key,value) VALUES
('library.scan_enabled','false'),
('library.scan_interval_minutes','120')
ON CONFLICT(key) DO NOTHING;

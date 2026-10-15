-- Inventory is independent of optional qBittorrent naming. Never inherit the
-- old import.rename_enabled=true: remote renaming requires fresh explicit opt-in.
INSERT OR IGNORE INTO settings(key,value)
SELECT 'library.source', CASE WHEN lower(trim(value))='external' THEN 'cinetta' ELSE 'local' END
FROM settings WHERE key='import.method';
INSERT OR IGNORE INTO settings(key,value) VALUES ('library.source','local');
INSERT OR IGNORE INTO settings(key,value) VALUES ('qbittorrent.rename_enabled','false');
UPDATE seed_policies SET cleanup_mode='remove_torrent_keep_files' WHERE cleanup_mode='remove_torrent_and_original';
-- A consistent private rollback copy is created by db::connect before this migration.
DELETE FROM settings WHERE key='storage.smb.connections';
CREATE TABLE torrent_archives (
  job_id INTEGER PRIMARY KEY,
  hash TEXT NOT NULL,
  save_path TEXT NOT NULL,
  torrent_data BLOB NOT NULL,
  mappings_json TEXT NOT NULL,
  completed INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE quality_profiles ADD COLUMN qbittorrent_category TEXT NOT NULL DEFAULT '';
ALTER TABLE quality_profiles ADD COLUMN qbittorrent_tags_template TEXT NOT NULL DEFAULT '[tracker]';
ALTER TABLE download_jobs ADD COLUMN qbittorrent_tags TEXT NOT NULL DEFAULT '';

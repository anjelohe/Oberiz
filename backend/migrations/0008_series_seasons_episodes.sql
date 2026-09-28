ALTER TABLE series ADD COLUMN monitor_mode TEXT NOT NULL DEFAULT 'all';
ALTER TABLE series ADD COLUMN metadata_synced_at TEXT;

CREATE TABLE IF NOT EXISTS series_seasons (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    series_id INTEGER NOT NULL,
    tmdb_season_id INTEGER,
    season_number INTEGER NOT NULL,
    name TEXT NOT NULL,
    overview TEXT,
    air_date TEXT,
    poster_path TEXT,
    episode_count INTEGER NOT NULL DEFAULT 0,
    monitored INTEGER NOT NULL DEFAULT 1,
    monitor_override INTEGER,
    quality_profile_id INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(series_id) REFERENCES series(id) ON DELETE CASCADE,
    FOREIGN KEY(quality_profile_id) REFERENCES quality_profiles(id) ON DELETE SET NULL,
    UNIQUE(series_id, season_number)
);

CREATE INDEX IF NOT EXISTS idx_series_seasons_series
ON series_seasons(series_id, season_number);

CREATE TABLE IF NOT EXISTS series_episodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    series_id INTEGER NOT NULL,
    season_id INTEGER NOT NULL,
    tmdb_episode_id INTEGER,
    season_number INTEGER NOT NULL,
    episode_number INTEGER NOT NULL,
    name TEXT NOT NULL,
    overview TEXT,
    air_date TEXT,
    still_path TEXT,
    runtime INTEGER,
    monitored INTEGER NOT NULL DEFAULT 1,
    monitor_override INTEGER,
    has_file INTEGER NOT NULL DEFAULT 0,
    quality_profile_id INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(series_id) REFERENCES series(id) ON DELETE CASCADE,
    FOREIGN KEY(season_id) REFERENCES series_seasons(id) ON DELETE CASCADE,
    FOREIGN KEY(quality_profile_id) REFERENCES quality_profiles(id) ON DELETE SET NULL,
    UNIQUE(series_id, season_number, episode_number)
);

CREATE INDEX IF NOT EXISTS idx_series_episodes_wanted
ON series_episodes(series_id, monitored, has_file, air_date);

CREATE TABLE IF NOT EXISTS episode_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    episode_id INTEGER NOT NULL,
    media_file_id INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(episode_id) REFERENCES series_episodes(id) ON DELETE CASCADE,
    FOREIGN KEY(media_file_id) REFERENCES media_files(id) ON DELETE CASCADE,
    UNIQUE(episode_id, media_file_id)
);

ALTER TABLE download_jobs ADD COLUMN season_number INTEGER;
ALTER TABLE download_jobs ADD COLUMN episode_number INTEGER;
ALTER TABLE download_jobs ADD COLUMN is_season_pack INTEGER NOT NULL DEFAULT 0;


CREATE TABLE IF NOT EXISTS series_target_state (
    series_id INTEGER NOT NULL,
    season_number INTEGER NOT NULL,
    episode_number INTEGER NOT NULL DEFAULT 0,
    last_search_at TEXT,
    last_grab_at TEXT,
    last_grab_title TEXT,
    last_grab_score INTEGER,
    last_error TEXT,
    status TEXT NOT NULL DEFAULT 'idle',
    PRIMARY KEY(series_id, season_number, episode_number),
    FOREIGN KEY(series_id) REFERENCES series(id) ON DELETE CASCADE
);


UPDATE settings
SET value='{Title} - S{Season:00}E{Episode:00} - {EpisodeTitle}'
WHERE key='import.series_template'
  AND value='{Title} - {Resolution} {Source} {Codec}';

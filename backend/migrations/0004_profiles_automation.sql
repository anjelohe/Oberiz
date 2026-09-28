CREATE TABLE IF NOT EXISTS language_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    allowed_languages_json TEXT NOT NULL DEFAULT '[]',
    scores_json TEXT NOT NULL DEFAULT '{}',
    allow_unknown INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS quality_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    media_type TEXT NOT NULL CHECK(media_type IN ('movie','series')),
    enabled INTEGER NOT NULL DEFAULT 1,
    upgrade_allowed INTEGER NOT NULL DEFAULT 1,
    cutoff_score INTEGER NOT NULL DEFAULT 500,
    min_seeders INTEGER NOT NULL DEFAULT 1,
    min_size_mb INTEGER,
    max_size_mb INTEGER,
    language_profile_id INTEGER,
    rules_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(language_profile_id) REFERENCES language_profiles(id) ON DELETE SET NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_quality_profiles_name_type
ON quality_profiles(media_type, name);

ALTER TABLE movies ADD COLUMN quality_profile_id INTEGER REFERENCES quality_profiles(id) ON DELETE SET NULL;
ALTER TABLE series ADD COLUMN quality_profile_id INTEGER REFERENCES quality_profiles(id) ON DELETE SET NULL;

CREATE TABLE IF NOT EXISTS automation_state (
    media_type TEXT NOT NULL CHECK(media_type IN ('movie','series')),
    media_id INTEGER NOT NULL,
    last_search_at TEXT,
    last_grab_at TEXT,
    last_grab_title TEXT,
    last_grab_score INTEGER,
    last_error TEXT,
    status TEXT NOT NULL DEFAULT 'idle',
    PRIMARY KEY(media_type, media_id)
);

INSERT INTO language_profiles (name, allowed_languages_json, scores_json, allow_unknown)
VALUES
('Any language', '["Spanish","Castellano","Latino","Dual","Multi","English"]', '{}', 1),
('Spanish preferred', '["Spanish","Castellano","Latino","Dual","Multi","English"]', '{"Spanish":140,"Castellano":140,"Dual":120,"Latino":80,"Multi":50,"English":0}', 1)
ON CONFLICT(name) DO NOTHING;

INSERT INTO quality_profiles (name, media_type, upgrade_allowed, cutoff_score, min_seeders, min_size_mb, max_size_mb, language_profile_id, rules_json)
SELECT 'Movies 1080p', 'movie', 1, 520, 2, 700, 25000, id,
'{"resolutions":{"1080P":260,"720P":80},"sources":{"REMUX":150,"BluRay":130,"WEB-DL":110,"WEBRip":70,"HDTV":20},"codecs":{"x265":35,"HEVC":35,"x264":15,"H.264":15,"AV1":25},"hdr":{"HDR":15,"HDR10+":25,"Dolby Vision":35},"audio":{"Atmos":25,"TrueHD":25,"DTS-HD":18,"DTS":8,"DD+":8,"AAC":0},"reject_terms":["CAM","TELESYNC","TS CAM","SCREENER"],"prefer_terms":{"PROPER":10,"REPACK":10},"allow_unknown_resolution":false,"allow_unknown_source":true,"series_prefer_pack":false,"series_accept_complete":true}'
FROM language_profiles WHERE name='Spanish preferred'
ON CONFLICT(media_type, name) DO NOTHING;

INSERT INTO quality_profiles (name, media_type, upgrade_allowed, cutoff_score, min_seeders, min_size_mb, max_size_mb, language_profile_id, rules_json)
SELECT 'Movies 4K', 'movie', 1, 760, 2, 2000, 100000, id,
'{"resolutions":{"2160P":420,"1080P":120},"sources":{"REMUX":210,"BluRay":165,"WEB-DL":140,"WEBRip":80},"codecs":{"x265":45,"HEVC":45,"AV1":35,"x264":5,"H.264":5},"hdr":{"HDR":45,"HDR10+":65,"Dolby Vision":85},"audio":{"Atmos":45,"TrueHD":40,"DTS-HD":30,"DTS":10,"DD+":12},"reject_terms":["CAM","TELESYNC","TS CAM","SCREENER","720P"],"prefer_terms":{"PROPER":10,"REPACK":10},"allow_unknown_resolution":false,"allow_unknown_source":true,"series_prefer_pack":false,"series_accept_complete":true}'
FROM language_profiles WHERE name='Spanish preferred'
ON CONFLICT(media_type, name) DO NOTHING;

INSERT INTO quality_profiles (name, media_type, upgrade_allowed, cutoff_score, min_seeders, min_size_mb, max_size_mb, language_profile_id, rules_json)
SELECT 'Movies Light', 'movie', 1, 440, 2, 400, 6500, id,
'{"resolutions":{"1080P":230,"720P":90},"sources":{"WEB-DL":120,"WEBRip":90,"BluRay":80},"codecs":{"x265":80,"HEVC":80,"AV1":70,"x264":10,"H.264":10},"hdr":{"HDR":5,"HDR10+":5,"Dolby Vision":5},"audio":{"AAC":10,"DD+":15},"reject_terms":["CAM","TELESYNC","REMUX","ISO"],"prefer_terms":{},"allow_unknown_resolution":false,"allow_unknown_source":true,"series_prefer_pack":false,"series_accept_complete":true}'
FROM language_profiles WHERE name='Any language'
ON CONFLICT(media_type, name) DO NOTHING;

INSERT INTO quality_profiles (name, media_type, upgrade_allowed, cutoff_score, min_seeders, min_size_mb, max_size_mb, language_profile_id, rules_json)
SELECT 'Series 1080p', 'series', 1, 500, 2, 150, 18000, id,
'{"resolutions":{"1080P":260,"720P":90},"sources":{"WEB-DL":150,"WEBRip":100,"BluRay":115,"HDTV":60},"codecs":{"x265":45,"HEVC":45,"x264":20,"H.264":20,"AV1":30},"hdr":{"HDR":10,"HDR10+":15,"Dolby Vision":20},"audio":{"DD+":15,"AAC":5,"DTS":8},"reject_terms":["CAM","TELESYNC"],"prefer_terms":{"PROPER":10,"REPACK":10},"allow_unknown_resolution":false,"allow_unknown_source":true,"series_prefer_pack":false,"series_accept_complete":true}'
FROM language_profiles WHERE name='Spanish preferred'
ON CONFLICT(media_type, name) DO NOTHING;

INSERT INTO quality_profiles (name, media_type, upgrade_allowed, cutoff_score, min_seeders, min_size_mb, max_size_mb, language_profile_id, rules_json)
SELECT 'Series 4K', 'series', 1, 700, 2, 500, 50000, id,
'{"resolutions":{"2160P":420,"1080P":120},"sources":{"WEB-DL":165,"BluRay":145,"REMUX":180,"WEBRip":80},"codecs":{"x265":50,"HEVC":50,"AV1":35},"hdr":{"HDR":45,"HDR10+":65,"Dolby Vision":85},"audio":{"Atmos":25,"TrueHD":25,"DD+":15},"reject_terms":["CAM","TELESYNC","720P"],"prefer_terms":{},"allow_unknown_resolution":false,"allow_unknown_source":true,"series_prefer_pack":false,"series_accept_complete":true}'
FROM language_profiles WHERE name='Spanish preferred'
ON CONFLICT(media_type, name) DO NOTHING;

INSERT INTO quality_profiles (name, media_type, upgrade_allowed, cutoff_score, min_seeders, min_size_mb, max_size_mb, language_profile_id, rules_json)
SELECT 'Anime', 'series', 1, 560, 1, 100, 30000, id,
'{"resolutions":{"1080P":270,"2160P":330,"720P":70},"sources":{"WEB-DL":145,"WEBRip":110,"BluRay":135},"codecs":{"x265":65,"HEVC":65,"AV1":45,"x264":15,"H.264":15},"hdr":{"HDR":10,"HDR10+":15,"Dolby Vision":15},"audio":{"AAC":15,"DD+":10,"FLAC":35},"reject_terms":["CAM","TELESYNC"],"prefer_terms":{"DUAL":70,"MULTI":45,"COMPLETE":30},"allow_unknown_resolution":false,"allow_unknown_source":true,"series_prefer_pack":true,"series_accept_complete":true}'
FROM language_profiles WHERE name='Any language'
ON CONFLICT(media_type, name) DO NOTHING;

UPDATE movies
SET quality_profile_id = (SELECT id FROM quality_profiles WHERE media_type='movie' AND name='Movies 1080p' LIMIT 1)
WHERE quality_profile_id IS NULL;

UPDATE series
SET quality_profile_id = (SELECT id FROM quality_profiles WHERE media_type='series' AND name='Series 1080p' LIMIT 1)
WHERE quality_profile_id IS NULL;

INSERT INTO settings(key, value)
VALUES ('automation.enabled','false')
ON CONFLICT(key) DO NOTHING;

INSERT INTO settings(key, value)
VALUES ('automation.interval_minutes','30')
ON CONFLICT(key) DO NOTHING;

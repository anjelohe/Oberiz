-- Refine only untouched built-in records. Existing user-customized profiles
-- and settings must remain exactly as they are during an upgrade.

DELETE FROM quality_profiles
WHERE media_type = 'movie'
  AND name = 'Movies Light'
  AND created_at = updated_at
  AND NOT EXISTS (
      SELECT 1 FROM movies WHERE quality_profile_id = quality_profiles.id
  );

UPDATE quality_profiles
SET cutoff_score = 0,
    min_seeders = 1,
    min_size_mb = 0,
    max_size_mb = NULL,
    language_profile_id = (SELECT id FROM language_profiles WHERE name = 'Spanish preferred'),
    request_quality = 'standard',
    rules_json = '{"resolutions":{"720P":10,"1080P":10},"sources":{"HDTV":0,"WEBRip":0,"WEB-DL":0},"codecs":{"x264":0,"H.264":0,"x265":0,"AV1":0,"HEVC":0},"hdr":{},"audio":{"DTS":0,"DTS-HD":0,"AAC":0,"Atmos":0,"DD+":0,"FLAC":0,"TrueHD":0},"reject_terms":["CAM","TELESYNC","TS CAM","SCREENER"],"prefer_terms":{"REPACK":10,"PROPER":10},"allow_unknown_resolution":false,"allow_unknown_source":true,"series_prefer_pack":false,"series_accept_complete":true}',
    updated_at = CURRENT_TIMESTAMP
WHERE media_type = 'movie' AND name = 'Movies 1080p' AND created_at = updated_at;

UPDATE quality_profiles
SET cutoff_score = 10,
    min_seeders = 1,
    min_size_mb = 0,
    max_size_mb = NULL,
    language_profile_id = (SELECT id FROM language_profiles WHERE name = 'Spanish preferred'),
    request_quality = '4k',
    rules_json = '{"resolutions":{"2160P":10},"sources":{"REMUX":0,"WEBRip":0,"HDTV":0,"BluRay":0,"WEB-DL":0},"codecs":{"x265":0,"HEVC":0,"x264":0,"H.264":0,"AV1":0},"hdr":{"HDR":0,"Dolby Vision":0,"HDR10+":0},"audio":{"DTS-HD":0,"DTS":0,"DD+":0,"TrueHD":0,"Atmos":0,"AAC":0,"FLAC":0},"reject_terms":["CAM","TELESYNC","TS CAM","SCREENER","720P"],"prefer_terms":{"REPACK":10,"PROPER":10},"allow_unknown_resolution":false,"allow_unknown_source":true,"series_prefer_pack":false,"series_accept_complete":true}',
    updated_at = CURRENT_TIMESTAMP
WHERE media_type = 'movie' AND name = 'Movies 4K' AND created_at = updated_at;

UPDATE quality_profiles
SET cutoff_score = 10,
    min_seeders = 1,
    min_size_mb = 0,
    max_size_mb = NULL,
    language_profile_id = (SELECT id FROM language_profiles WHERE name = 'Any language'),
    request_quality = '4k',
    rules_json = '{"resolutions":{"720P":10,"2160P":20,"1080P":10},"sources":{"WEB-DL":0,"WEBRip":0,"HDTV":0,"BluRay":0},"codecs":{"x264":0,"H.264":0,"x265":0,"HEVC":0,"AV1":0},"hdr":{},"audio":{"Atmos":0,"TrueHD":0,"DTS":0,"DTS-HD":0,"AAC":0,"FLAC":0,"DD+":0},"reject_terms":["CAM","TELESYNC"],"prefer_terms":{"DUAL":70,"COMPLETE":30,"MULTI":45},"allow_unknown_resolution":false,"allow_unknown_source":true,"series_prefer_pack":true,"series_accept_complete":true}',
    updated_at = CURRENT_TIMESTAMP
WHERE media_type = 'series' AND name = 'Anime' AND created_at = updated_at;

UPDATE quality_profiles
SET cutoff_score = 10,
    min_seeders = 1,
    min_size_mb = 0,
    max_size_mb = NULL,
    language_profile_id = (SELECT id FROM language_profiles WHERE name = 'Spanish preferred'),
    request_quality = 'standard',
    rules_json = '{"resolutions":{"720P":10,"1080P":10},"sources":{"HDTV":0,"WEB-DL":0,"WEBRip":0},"codecs":{"H.264":0,"x265":0,"AV1":0,"x264":0,"HEVC":0},"hdr":{},"audio":{"DD+":0,"AAC":0,"DTS":0,"DTS-HD":0,"FLAC":0,"Atmos":0,"TrueHD":0},"reject_terms":["CAM","TELESYNC"],"prefer_terms":{"PROPER":10,"REPACK":10},"allow_unknown_resolution":false,"allow_unknown_source":true,"series_prefer_pack":false,"series_accept_complete":true}',
    updated_at = CURRENT_TIMESTAMP
WHERE media_type = 'series' AND name = 'Series 1080p' AND created_at = updated_at;

UPDATE quality_profiles
SET cutoff_score = 10,
    min_seeders = 1,
    min_size_mb = 0,
    max_size_mb = NULL,
    language_profile_id = (SELECT id FROM language_profiles WHERE name = 'Spanish preferred'),
    request_quality = '4k',
    rules_json = '{"resolutions":{"2160P":10},"sources":{"REMUX":0,"BluRay":0,"WEB-DL":0,"HDTV":0,"WEBRip":0},"codecs":{"AV1":0,"HEVC":0,"x264":0,"x265":0,"H.264":0},"hdr":{"Dolby Vision":0,"HDR":0,"HDR10+":0},"audio":{"FLAC":0,"DTS-HD":0,"TrueHD":0,"DTS":0,"AAC":0,"Atmos":0,"DD+":0},"reject_terms":["CAM","TELESYNC","720P"],"prefer_terms":{},"allow_unknown_resolution":false,"allow_unknown_source":true,"series_prefer_pack":false,"series_accept_complete":true}',
    updated_at = CURRENT_TIMESTAMP
WHERE media_type = 'series' AND name = 'Series 4K' AND created_at = updated_at;

UPDATE settings SET value = 'false' WHERE key = 'import.rename_enabled' AND value = 'true';
UPDATE settings SET value = 'false' WHERE key = 'import.cleanup_after_seed' AND value = 'true';
UPDATE settings SET value = '{Title} - S{Season:00}E{Episode:00} - {EpisodeTitle}'
WHERE key = 'import.series_template' AND value = '{Title} - {Resolution} {Source} {Codec}';
UPDATE settings SET value = 'true' WHERE key = 'rss.enabled' AND value = 'false';
UPDATE settings SET value = '30' WHERE key = 'rss.interval_minutes' AND value = '15';

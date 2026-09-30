-- A series pack can legitimately be much larger than one episode.  Existing
-- max_size_mb remains the general fallback; this optional value overrides it
-- only for season and complete-series releases.
ALTER TABLE quality_profiles ADD COLUMN max_season_pack_size_mb INTEGER;

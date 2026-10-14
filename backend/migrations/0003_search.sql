ALTER TABLE release_cache ADD COLUMN details_url TEXT;
ALTER TABLE release_cache ADD COLUMN media_type TEXT;
ALTER TABLE release_cache ADD COLUMN query_text TEXT;

CREATE INDEX IF NOT EXISTS idx_release_cache_query
ON release_cache(query_text, media_type, created_at DESC);

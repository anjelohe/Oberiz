-- Releases the user has explicitly rejected for one movie or series, so
-- neither the scheduled search, RSS nor the manual search offers them again.
-- release_key is the release title lowercased with every run of
-- non-alphanumeric characters collapsed to one space: search results carry no
-- infohash up front, and the same release is usually published under the same
-- title by several indexers, so the title is the identity both a search result
-- and a past download job can be matched on.
CREATE TABLE rejected_releases (
  id INTEGER PRIMARY KEY,
  media_type TEXT NOT NULL CHECK(media_type IN ('movie','series')),
  media_id INTEGER NOT NULL,
  release_key TEXT NOT NULL,
  release_title TEXT NOT NULL,
  indexer_name TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(media_type, media_id, release_key)
);

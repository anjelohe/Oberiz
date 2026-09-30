-- Native clients may request a subset of a series instead of every missing episode.
ALTER TABLE media_requests ADD COLUMN requested_seasons TEXT;
ALTER TABLE media_requests ADD COLUMN monitor_future_seasons INTEGER NOT NULL DEFAULT 0;

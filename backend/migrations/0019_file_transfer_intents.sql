-- Durable record of "about to move/copy/hardlink this file" written before
-- the transfer runs, not only the mappings_json snapshot written after one
-- succeeds. A "move" deletes its source as it goes: if the process crashes
-- between a transfer actually completing and that mapping being persisted,
-- the source file is already gone (so it will never be rediscovered and
-- retried) and nothing recorded that the transfer had, in fact, succeeded —
-- the file sits in the library with no episode/reseed association. On the
-- next import attempt for the same job, a leftover row here is checked
-- against the real filesystem state before anything else runs, so that gap
-- can be closed instead of silently losing the mapping.
CREATE TABLE file_transfer_intents (
  id INTEGER PRIMARY KEY,
  job_id INTEGER NOT NULL,
  original_rel TEXT NOT NULL,
  library_rel TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(job_id, library_rel)
);

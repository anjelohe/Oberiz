-- The old plain index (0012) never stopped two concurrent POSTs with the
-- same client_name/client_request_id from both passing the
-- SELECT-then-INSERT check in create_request_internal and creating two
-- rows: idempotency was a convention in application code, not a guarantee
-- the database enforced. Keep the newest row for any pair that already has
-- duplicates before adding the real constraint, so this migration doesn't
-- fail on a database that already has them.
DELETE FROM media_requests
WHERE client_name IS NOT NULL AND client_request_id IS NOT NULL
  AND id NOT IN (
    SELECT MAX(id) FROM media_requests
    WHERE client_name IS NOT NULL AND client_request_id IS NOT NULL
    GROUP BY client_name, client_request_id
  );

DROP INDEX IF EXISTS idx_media_requests_client_request;

-- SQLite treats NULLs in a UNIQUE index as distinct from each other, so
-- requests from clients that don't send an idempotency key (client_name
-- and/or client_request_id NULL) are unaffected by this constraint.
CREATE UNIQUE INDEX idx_media_requests_client_request
ON media_requests(client_name, client_request_id);

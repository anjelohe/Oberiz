-- Native-client metadata is optional so existing public and compatibility clients keep working.
ALTER TABLE media_requests ADD COLUMN client_request_id TEXT;
ALTER TABLE media_requests ADD COLUMN client_name TEXT;

CREATE INDEX IF NOT EXISTS idx_media_requests_client_request
ON media_requests(client_name, client_request_id);

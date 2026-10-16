-- Independent reports. No foreign keys or writes to the managed media library.
CREATE TABLE missing_reports (
    id TEXT PRIMARY KEY,
    signature TEXT NOT NULL,
    state TEXT NOT NULL,
    report_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX missing_reports_scope ON missing_reports(signature,created_at DESC);
CREATE TABLE missing_report_actions (
    action_key TEXT PRIMARY KEY,
    signature TEXT NOT NULL,
    job_id TEXT NOT NULL REFERENCES missing_reports(id) ON DELETE CASCADE
);

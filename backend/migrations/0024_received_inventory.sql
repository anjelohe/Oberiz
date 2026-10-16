CREATE TABLE received_library_inventory (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    received_at TEXT NOT NULL,
    items_json TEXT NOT NULL
);
DELETE FROM settings WHERE key IN ('cinetta.url', 'cinetta.inventory_key');

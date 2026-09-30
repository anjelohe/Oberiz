-- Classifies profiles exposed to native request clients such as Cinetta.
-- Existing 4K/2160 profiles keep their intended routing after the migration.
ALTER TABLE quality_profiles ADD COLUMN request_quality TEXT NOT NULL DEFAULT 'standard';

UPDATE quality_profiles
SET request_quality = '4k'
WHERE UPPER(name) LIKE '%4K%'
   OR UPPER(name) LIKE '%2160%'
   OR UPPER(rules_json) LIKE '%2160P%';

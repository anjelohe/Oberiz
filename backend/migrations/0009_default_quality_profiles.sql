ALTER TABLE quality_profiles ADD COLUMN is_default INTEGER NOT NULL DEFAULT 0;

UPDATE quality_profiles
SET is_default=1
WHERE id=(
  SELECT id FROM quality_profiles
  WHERE media_type='movie' AND name='Movies 1080p'
  LIMIT 1
)
AND NOT EXISTS(
  SELECT 1 FROM quality_profiles WHERE media_type='movie' AND is_default=1
);

UPDATE quality_profiles
SET is_default=1
WHERE id=(
  SELECT id FROM quality_profiles
  WHERE media_type='series' AND name='Series 1080p'
  LIMIT 1
)
AND NOT EXISTS(
  SELECT 1 FROM quality_profiles WHERE media_type='series' AND is_default=1
);

UPDATE quality_profiles
SET is_default=1
WHERE id=(
  SELECT id FROM quality_profiles
  WHERE media_type='movie' AND enabled=1
  ORDER BY id LIMIT 1
)
AND NOT EXISTS(
  SELECT 1 FROM quality_profiles WHERE media_type='movie' AND is_default=1
);

UPDATE quality_profiles
SET is_default=1
WHERE id=(
  SELECT id FROM quality_profiles
  WHERE media_type='series' AND enabled=1
  ORDER BY id LIMIT 1
)
AND NOT EXISTS(
  SELECT 1 FROM quality_profiles WHERE media_type='series' AND is_default=1
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_quality_profiles_one_default_per_type
ON quality_profiles(media_type)
WHERE is_default=1;

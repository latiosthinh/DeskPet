-- DeskPet site counters. One row per pet, kind and UTC day.
-- kind: 'visit' (detail page view), 'download' (module download), 'update' (app update check).
CREATE TABLE IF NOT EXISTS counts (
  pet_id TEXT NOT NULL,
  kind   TEXT NOT NULL,
  day    TEXT NOT NULL,          -- YYYY-MM-DD (UTC)
  n      INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (pet_id, kind, day)
);

CREATE INDEX IF NOT EXISTS counts_day ON counts (day);

-- Increment (used by the functions):
-- INSERT INTO counts (pet_id, kind, day, n) VALUES (?1, ?2, ?3, 1)
--   ON CONFLICT (pet_id, kind, day) DO UPDATE SET n = n + 1;

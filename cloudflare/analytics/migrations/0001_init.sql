CREATE TABLE IF NOT EXISTS desktop_daily_active (
  installation_hash TEXT NOT NULL,
  activity_date TEXT NOT NULL,
  event_name TEXT NOT NULL,
  app_version TEXT NOT NULL,
  platform TEXT NOT NULL CHECK (platform IN ('macos', 'windows')),
  PRIMARY KEY (installation_hash, activity_date, event_name)
) WITHOUT ROWID;

CREATE INDEX IF NOT EXISTS idx_desktop_daily_active_date
  ON desktop_daily_active (activity_date);

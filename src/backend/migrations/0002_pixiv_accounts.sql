CREATE TABLE IF NOT EXISTS pixiv_accounts (
  user_uid TEXT PRIMARY KEY,
  user_name TEXT,
  cookie_json TEXT NOT NULL,
  is_active INTEGER NOT NULL DEFAULT 0 CHECK (is_active IN (0, 1)),
  last_verified_at TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_pixiv_accounts_active
  ON pixiv_accounts (is_active)
  WHERE is_active = 1;

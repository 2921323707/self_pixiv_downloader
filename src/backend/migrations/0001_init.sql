CREATE TABLE IF NOT EXISTS images (
  image_id TEXT PRIMARY KEY,
  pixiv_id TEXT NOT NULL,
  page_index INTEGER NOT NULL DEFAULT 0,
  author_uid TEXT,
  title TEXT,
  category TEXT NOT NULL CHECK (category IN ('normal', 'r18', 'nsfw')),
  local_path TEXT NOT NULL,
  thumbnail_path TEXT,
  width INTEGER,
  height INTEGER,
  map_x REAL,
  map_y REAL,
  downloaded_at TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_images_pixiv_page
  ON images (pixiv_id, page_index);

CREATE INDEX IF NOT EXISTS idx_images_category_created_at
  ON images (category, created_at);

CREATE INDEX IF NOT EXISTS idx_images_author_uid
  ON images (author_uid);

CREATE TABLE IF NOT EXISTS image_tags (
  image_id TEXT NOT NULL,
  tag TEXT NOT NULL,
  tag_locale TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (image_id) REFERENCES images(image_id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_image_tags_image_tag
  ON image_tags (image_id, tag);

CREATE INDEX IF NOT EXISTS idx_image_tags_tag
  ON image_tags (tag);

CREATE TABLE IF NOT EXISTS image_sources (
  image_id TEXT NOT NULL,
  source TEXT NOT NULL CHECK (source IN ('single', 'bookmark', 'author', 'top10', 'random', 'smart')),
  task_id TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (image_id) REFERENCES images(image_id) ON DELETE CASCADE,
  FOREIGN KEY (task_id) REFERENCES tasks(task_id) ON DELETE SET NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_image_sources_identity
  ON image_sources (image_id, source, COALESCE(task_id, ''));

CREATE INDEX IF NOT EXISTS idx_image_sources_source
  ON image_sources (source);

CREATE TABLE IF NOT EXISTS tasks (
  task_id TEXT PRIMARY KEY,
  type TEXT NOT NULL CHECK (type IN ('single', 'bookmark', 'author', 'top10', 'random', 'smart')),
  status TEXT NOT NULL CHECK (status IN ('pending', 'running', 'completed', 'completed_with_errors', 'failed', 'cancelled')),
  request_json TEXT NOT NULL,
  progress_total INTEGER,
  progress_done INTEGER NOT NULL DEFAULT 0 CHECK (progress_done >= 0),
  progress_failed INTEGER NOT NULL DEFAULT 0 CHECK (progress_failed >= 0),
  current_item TEXT,
  error_code TEXT,
  error_message TEXT,
  created_at TEXT NOT NULL,
  started_at TEXT,
  finished_at TEXT,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tasks_status_created_at
  ON tasks (status, created_at);

CREATE INDEX IF NOT EXISTS idx_tasks_type_created_at
  ON tasks (type, created_at);

CREATE TABLE IF NOT EXISTS task_logs (
  log_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  level TEXT NOT NULL CHECK (level IN ('debug', 'info', 'warn', 'error')),
  phase TEXT NOT NULL,
  message TEXT NOT NULL,
  context_json TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (task_id) REFERENCES tasks(task_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_task_logs_task_created_at
  ON task_logs (task_id, created_at);

CREATE TABLE IF NOT EXISTS task_items (
  item_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  pixiv_id TEXT,
  page_index INTEGER,
  status TEXT NOT NULL CHECK (status IN ('discovered', 'duplicate_skipped', 'downloading', 'saved', 'item_failed', 'policy_skipped')),
  image_id TEXT,
  error_code TEXT,
  error_message TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (task_id) REFERENCES tasks(task_id) ON DELETE CASCADE,
  FOREIGN KEY (image_id) REFERENCES images(image_id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_task_items_task_status
  ON task_items (task_id, status);

CREATE INDEX IF NOT EXISTS idx_task_items_pixiv_page
  ON task_items (pixiv_id, page_index);

CREATE TABLE IF NOT EXISTS smart_retrievals (
  retrieval_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  user_prompt TEXT NOT NULL,
  llm_model TEXT NOT NULL,
  llm_output_json TEXT NOT NULL,
  tags_json TEXT NOT NULL,
  negative_tags_json TEXT NOT NULL,
  requested_count INTEGER NOT NULL CHECK (requested_count > 0),
  r18_policy TEXT NOT NULL CHECK (r18_policy IN ('exclude', 'include_blurred', 'include_visible', 'only_r18')),
  created_at TEXT NOT NULL,
  FOREIGN KEY (task_id) REFERENCES tasks(task_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_smart_retrievals_task_id
  ON smart_retrievals (task_id);

CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value_json TEXT NOT NULL,
  is_secret INTEGER NOT NULL DEFAULT 0 CHECK (is_secret IN (0, 1)),
  updated_at TEXT NOT NULL
);

INSERT OR IGNORE INTO settings (key, value_json, is_secret, updated_at)
VALUES
  ('download_base_path', '"project:output"', 0, datetime('now')),
  ('deepseek_base_url', '"https://api.deepseek.com"', 0, datetime('now')),
  ('deepseek_model', '"deepseek-v4-flash"', 0, datetime('now')),
  ('default_batch_count', '20', 0, datetime('now')),
  ('max_request_count', '100', 0, datetime('now')),
  ('r18_policy', '"exclude"', 0, datetime('now')),
  ('theme_id', '"cyan-studio"', 0, datetime('now'));

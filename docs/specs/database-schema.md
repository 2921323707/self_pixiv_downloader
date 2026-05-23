# Database Schema Specification

Storage: SQLite

Requirements: `REQ-IMG-001`, `REQ-DL-006`, `REQ-TASK-002`, `REQ-AI-003`, `REQ-CFG-001` through `REQ-CFG-007`

## Design Principles

- SQLite is the local source of truth for image indexes, tasks, settings, and provenance.
- Local filesystem is the source of truth for image bytes.
- All records use explicit timestamps for auditability.
- Batch operations should preserve item-level history rather than only final task status.

## Tables

### `images`

| Column | Type | Constraint |
| --- | --- | --- |
| `image_id` | TEXT | Primary key |
| `pixiv_id` | TEXT | Not null |
| `page_index` | INTEGER | Not null default 0 |
| `author_uid` | TEXT | Nullable |
| `title` | TEXT | Nullable |
| `category` | TEXT | Not null |
| `local_path` | TEXT | Not null |
| `thumbnail_path` | TEXT | Nullable |
| `width` | INTEGER | Nullable |
| `height` | INTEGER | Nullable |
| `map_x` | REAL | Nullable |
| `map_y` | REAL | Nullable |
| `downloaded_at` | TEXT | Not null |
| `created_at` | TEXT | Not null |
| `updated_at` | TEXT | Not null |

Unique index: `(pixiv_id, page_index)`.

### `image_tags`

| Column | Type | Constraint |
| --- | --- | --- |
| `image_id` | TEXT | Not null, foreign key |
| `tag` | TEXT | Not null |
| `tag_locale` | TEXT | Nullable |
| `created_at` | TEXT | Not null |

Unique index: `(image_id, tag)`.

Rationale: normalized tags support faster filtering than a JSON-only field. API may still return tags as arrays.

### `image_sources`

Tracks provenance when the same image is encountered by multiple flows.

| Column | Type | Constraint |
| --- | --- | --- |
| `image_id` | TEXT | Not null, foreign key |
| `source` | TEXT | Not null |
| `task_id` | TEXT | Nullable |
| `created_at` | TEXT | Not null |

Unique index: `(image_id, source, task_id)`.

### `tasks`

| Column | Type | Constraint |
| --- | --- | --- |
| `task_id` | TEXT | Primary key |
| `type` | TEXT | Not null |
| `status` | TEXT | Not null |
| `request_json` | TEXT | Not null |
| `progress_total` | INTEGER | Nullable |
| `progress_done` | INTEGER | Not null default 0 |
| `progress_failed` | INTEGER | Not null default 0 |
| `current_item` | TEXT | Nullable |
| `error_code` | TEXT | Nullable |
| `error_message` | TEXT | Nullable |
| `created_at` | TEXT | Not null |
| `started_at` | TEXT | Nullable |
| `finished_at` | TEXT | Nullable |
| `updated_at` | TEXT | Not null |

Indexes:

- `(status, created_at)`
- `(type, created_at)`

### `task_logs`

| Column | Type | Constraint |
| --- | --- | --- |
| `log_id` | TEXT | Primary key |
| `task_id` | TEXT | Not null, foreign key |
| `level` | TEXT | Not null |
| `phase` | TEXT | Not null |
| `message` | TEXT | Not null |
| `context_json` | TEXT | Nullable |
| `created_at` | TEXT | Not null |

Index: `(task_id, created_at)`.

### `task_items`

Item-level state for batch traceability.

| Column | Type | Constraint |
| --- | --- | --- |
| `item_id` | TEXT | Primary key |
| `task_id` | TEXT | Not null, foreign key |
| `pixiv_id` | TEXT | Nullable |
| `page_index` | INTEGER | Nullable |
| `status` | TEXT | Not null |
| `image_id` | TEXT | Nullable, foreign key |
| `error_code` | TEXT | Nullable |
| `error_message` | TEXT | Nullable |
| `created_at` | TEXT | Not null |
| `updated_at` | TEXT | Not null |

Indexes:

- `(task_id, status)`
- `(pixiv_id, page_index)`

### `smart_retrievals`

| Column | Type | Constraint |
| --- | --- | --- |
| `retrieval_id` | TEXT | Primary key |
| `task_id` | TEXT | Not null, foreign key |
| `user_prompt` | TEXT | Not null |
| `llm_model` | TEXT | Not null |
| `llm_output_json` | TEXT | Not null |
| `tags_json` | TEXT | Not null |
| `negative_tags_json` | TEXT | Not null |
| `requested_count` | INTEGER | Not null |
| `r18_policy` | TEXT | Not null |
| `created_at` | TEXT | Not null |

Index: `(task_id)`.

### `settings`

Single-row key/value configuration.

| Column | Type | Constraint |
| --- | --- | --- |
| `key` | TEXT | Primary key |
| `value_json` | TEXT | Not null |
| `is_secret` | INTEGER | Not null default 0 |
| `updated_at` | TEXT | Not null |

Secret values are stored locally. API responses must mask secret values.

## Initial Settings Keys

| Key | Default |
| --- | --- |
| `pixiv_cookie` | unset |
| `download_base_path` | `project:output` |
| `deepseek_api_key` | unset |
| `deepseek_base_url` | `https://api.deepseek.com` |
| `deepseek_model` | `deepseek-v4-flash` |
| `max_request_count` | `100` |
| `default_batch_count` | `20` |
| `r18_policy` | `exclude` |
| `theme_id` | `cyan-studio` |

## Migration Strategy

Use numbered SQL migrations in backend implementation:

- `0001_init.sql`: tables and indexes above.
- Later migrations must be additive when possible.
- Any schema change should update this document and `traceability.md`.

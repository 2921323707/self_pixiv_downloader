# API Contract Specification

Backend: Rust + Axum

Frontend: Next.js

Base path: `/api`

## Response Envelope

Successful responses:

```json
{
  "data": {},
  "trace_id": "request-trace-id"
}
```

Errors:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Readable message",
    "details": {}
  },
  "trace_id": "request-trace-id"
}
```

## Download APIs

### `POST /api/download/single`

Requirement: `REQ-DL-001`

Compatibility note: the current Axum wrapper also accepts `POST /api/downloads/single` as a plural alias.

Request:

```json
{
  "pixiv_id": "123456",
  "page_index": 0,
  "r18_policy": "exclude"
}
```

Response: `202 Accepted`

```json
{
  "data": {
    "task_id": "task_uuid",
    "image_id": null,
    "download_status": "pending"
  }
}
```

The task is executed by the background worker after enqueue. Clients should poll `GET /api/tasks/{task_id}` for `running`, `completed`, or `failed` state and final item/image fields.

### `POST /api/downloads/bookmarks`

Requirement: `REQ-DL-002`

Implemented in Phase 5B as the current-user bookmark batch vertical slice.

Request:

```json
{
  "limit": 20,
  "r18_policy": "include_blurred"
}
```

`limit` is optional. If omitted, backend uses `default_batch_count` when available, otherwise `20`. The final limit must not exceed `max_request_count`.

### `POST /api/downloads/author`

Requirement: `REQ-DL-003`

Implemented in Phase 5A as the first batch download vertical slice.

Request:

```json
{
  "author_uid": "98765",
  "limit": 20,
  "r18_policy": "exclude"
}
```

`limit` and `r18_policy` are optional. If omitted, backend uses `default_batch_count` and settings `r18_policy`. The final limit must not exceed `max_request_count`.

Response: `202 Accepted`

```json
{
  "data": {
    "task_id": "task_uuid",
    "download_status": "pending"
  }
}
```

The worker discovers works for the author, creates/updates task items, reuses DB-aware image download behavior, and may finish as `completed_with_errors` when some items fail.

### `POST /api/downloads/top10`

Requirement: `REQ-DL-004`

Request:

```json
{
  "mode": "refresh_only"
}
```

`mode` values:

- `refresh_only`
- `download_all`

### `POST /api/downloads/random`

Requirement: `REQ-DL-005`

Request:

```json
{
  "limit": 1,
  "r18_policy": "exclude"
}
```

Initial V1 random defaults to one work. Optional multi-random can reuse the shared batch limit policy.

## Smart Retrieval APIs

### `POST /api/smart/parse`

Requirement: `REQ-AI-001`

Parses natural language without starting a download.

Request:

```json
{
  "prompt": "下载一些蓝色头发、赛博朋克风格的少女插画",
  "r18_policy": "exclude",
  "count": null
}
```

Response:

```json
{
  "data": {
    "tags": ["blue hair", "cyberpunk", "girl"],
    "negative_tags": [],
    "count_recommend": 20,
    "r18_policy": "exclude",
    "confidence": 0.82,
    "model": "deepseek-v4-flash"
  }
}
```

### `POST /api/smart/download`

Requirements: `REQ-AI-002`, `REQ-AI-003`, `REQ-AI-004`

Implemented in Phase 6B. Creates a `smart` task from an accepted tag plan, searches Pixiv by tags, downloads discovered works through the shared DB-aware downloader, and persists prompt/model/tag provenance in `smart_retrievals`.

Request:

```json
{
  "prompt": "下载一些蓝色头发、赛博朋克风格的少女插画",
  "tags": ["青髪", "サイバーパンク", "女の子"],
  "negative_tags": [],
  "count": 20,
  "r18_policy": "exclude",
  "model": "deepseek-v4-flash"
}
```

`count` follows the same safety policy as batch endpoints: request count first, then `default_batch_count`, always capped by `max_request_count`. Empty tags are rejected. Pixiv cookie is required from settings or `PIXIV_PHPSESSID`.

Response: `202 Accepted`

```json
{
  "data": {
    "task_id": "task_uuid",
    "download_status": "pending"
  }
}
```

## Task APIs

### `GET /api/tasks`

Requirements: `REQ-TASK-002`, `REQ-UI-003`

Query:

- `status`
- `type`
- `limit`
- `cursor`

Response:

```json
{
  "data": {
    "items": [
      {
        "task_id": "task_uuid",
        "type": "single",
        "status": "completed",
        "progress_total": 1,
        "progress_done": 1,
        "progress_failed": 0,
        "current_item": null,
        "error_code": null,
        "error_message": null,
        "created_at": "2026-05-22T10:00:00Z",
        "started_at": "2026-05-22T10:00:01Z",
        "finished_at": "2026-05-22T10:00:02Z",
        "updated_at": "2026-05-22T10:00:02Z"
      }
    ],
    "next_cursor": null
  }
}
```

### `GET /api/tasks/{task_id}`

Requirements: `REQ-TASK-002`, `REQ-TASK-004`

Response:

```json
{
  "data": {
    "task_id": "task_uuid",
    "type": "smart",
    "status": "running",
    "progress_total": 20,
    "progress_done": 4,
    "progress_failed": 1,
    "current_item": "123456",
    "error_code": null,
    "error_message": null,
    "created_at": "2026-05-21T10:00:00Z",
    "started_at": "2026-05-21T10:00:02Z",
      "finished_at": null,
      "items": [],
      "logs": []
  }
}
```

### `POST /api/tasks/{task_id}/cancel`

Requirement: `REQ-TASK-003`

Optional V1 endpoint if cancellation is implemented.

## Gallery APIs

### `GET /api/images`

Requirements: `REQ-IMG-002`, `REQ-IMG-003`, `REQ-UI-005`

Query:

- `tag`
- `category`
- `author_uid`
- `source`
- `r18_visibility`: `exclude` (default), `include`, `only_r18`
- `limit`
- `cursor`

Response:

```json
{
  "data": {
    "items": [
      {
        "image_id": "image_uuid",
        "pixiv_id": "123456",
        "page_index": 0,
        "title": "title",
        "author_uid": "98765",
        "tags": ["tag"],
        "category": "normal",
        "thumbnail_url": "/api/images/image_uuid/file",
        "preview_url": "/api/images/image_uuid/file",
        "width": 1200,
        "height": 1800,
        "downloaded_at": "2026-05-22T10:00:00Z",
        "created_at": "2026-05-22T10:00:00Z"
      }
    ],
    "next_cursor": null
  }
}
```

Implementation note: Phase 4D returns secure preview URLs that point to the local-file serving endpoint. Generated thumbnail files can be added later.

### `GET /api/images/{image_id}`

Requirement: `REQ-IMG-004`

Returns full metadata for preview panel.

Current Phase 4B response includes metadata, tags, and source history, but still omits local filesystem paths:

```json
{
  "data": {
    "image_id": "image_uuid",
    "pixiv_id": "123456",
    "page_index": 0,
    "title": "title",
    "author_uid": "98765",
    "tags": ["tag"],
    "sources": [
      {
        "source": "single",
        "task_id": "task_uuid",
        "created_at": "2026-05-22T10:00:00Z"
      }
    ],
    "category": "normal",
    "thumbnail_url": "/api/images/image_uuid/file",
    "preview_url": "/api/images/image_uuid/file",
    "width": 1200,
    "height": 1800,
    "map_x": null,
    "map_y": null,
    "downloaded_at": "2026-05-22T10:00:00Z",
    "created_at": "2026-05-22T10:00:00Z",
    "updated_at": "2026-05-22T10:00:00Z"
  }
}
```

## Settings APIs

### `GET /api/settings`

Requirements: `REQ-CFG-001`, `REQ-SEC-001`, `REQ-UI-004`

Returns public settings. Secret values are masked as `"***"`.

```json
{
  "data": {
    "items": [
      {
        "key": "theme_id",
        "value": "cyan-studio",
        "is_secret": false,
        "updated_at": "2026-05-22T10:00:00Z"
      },
      {
        "key": "deepseek_api_key",
        "value": "***",
        "is_secret": true,
        "updated_at": "2026-05-22T10:00:00Z"
      }
    ]
  }
}
```

### `PUT /api/settings/{key}`

Requirements: `REQ-CFG-001`, `REQ-SEC-001`, `REQ-UI-004`

Saves a known setting through repository-level validation. Unknown keys are rejected. Secret settings are stored raw in SQLite but returned masked; sending `"***"` for an existing secret keeps the previous raw value.

Request:

```json
{
  "value": "include_blurred"
}
```

Response:

```json
{
  "data": {
    "key": "r18_policy",
    "value": "include_blurred",
    "is_secret": false,
    "updated_at": "2026-05-22T10:00:00Z"
  }
}
```

### `GET /api/images/{image_id}/file`

Requirement: `REQ-IMG-004`

Implemented in Phase 4D. Serves local image bytes without exposing the local filesystem path.

Minimum behavior:

- Validates `image_id` through the image repository.
- Returns `404` for unknown image ids or missing files.
- Returns a stable error for unsafe or invalid stored paths.
- Sets an image `Content-Type` based on the stored file extension or detected bytes.
- Does not return local filesystem paths in JSON responses.

Initial `thumbnail_url` / `preview_url` point at this secure endpoint. A separate generated-thumbnail endpoint can be added after the preview flow is stable.

### `DELETE /api/images/{image_id}`

Requirement: `REQ-IMG-007`

Implemented in Phase 4E. Deletes one indexed image by `image_id`.

Minimum behavior:

- Resolves the stored `local_path` only under the configured download root or startup download root.
- Deletes the local file when present.
- If the file is already missing but its parent directory is still inside an allowed root, removes the stale SQLite index.
- Deletes the `images` row; `image_tags` and `image_sources` cascade, while existing `task_items.image_id` is set to null.
- Does not expose local filesystem paths in JSON responses.

Response:

```json
{
  "data": {
    "image_id": "image_uuid",
    "status": "deleted",
    "pixiv_id": "123456",
    "page_index": 0,
    "file_deleted": true,
    "file_missing": false,
    "error_code": null,
    "error_message": null
  }
}
```

### `POST /api/images/delete-batch`

Requirement: `REQ-IMG-007`

Implemented in Phase 4E. Deletes up to 100 selected images and returns per-item results so partial failures are visible to the frontend.

Request:

```json
{
  "image_ids": ["image_uuid_1", "image_uuid_2"]
}
```

Response:

```json
{
  "data": {
    "items": [
      {
        "image_id": "image_uuid_1",
        "status": "deleted",
        "pixiv_id": "123456",
        "page_index": 0,
        "file_deleted": true,
        "file_missing": false,
        "error_code": null,
        "error_message": null
      }
    ],
    "deleted_count": 1,
    "failed_count": 0
  }
}
```

### `PATCH /api/images/{image_id}`

Requirement: `REQ-IMG-006`

Request:

```json
{
  "category": "normal",
  "tags": ["tag1", "tag2"],
  "map_x": 0.42,
  "map_y": 0.77
}
```

## Map APIs

### `GET /api/map/points`

Requirement: `REQ-IMG-005`

Query:

- `tag`
- `category`
- `r18_visibility`

Returns points for visual exploration.

## Future Settings APIs

### `PATCH /api/settings`

Requirements: `REQ-CFG-001` through `REQ-CFG-006`, `REQ-SEC-001`

Planned bulk settings update endpoint. Phase 4B currently implements key-scoped `PUT /api/settings/{key}` instead.

Request:

```json
{
  "download_base_path": "~/Downloads/Pixiv Platform",
  "deepseek_base_url": "https://api.deepseek.com",
  "default_batch_count": 20,
  "max_request_count": 100,
  "r18_policy": "exclude",
  "theme_id": "cyan-studio"
}
```

### `PUT /api/settings/secrets/pixiv-cookie`

Requirement: `REQ-CFG-001`

Planned dedicated secret endpoint. Phase 4B uses `PUT /api/settings/pixiv_cookie` and always returns masked values.

### `PUT /api/settings/secrets/deepseek-key`

Requirement: `REQ-CFG-003`

Planned dedicated secret endpoint. Phase 4B uses `PUT /api/settings/deepseek_api_key` and always returns masked values.

### `POST /api/settings/test/pixiv`

Tests Pixiv cookie validity without exposing the cookie.

Current Phase 4C behavior:

- Reads `pixiv_cookie` from settings, with `PIXIV_PHPSESSID` as runtime fallback.
- Returns only status/work metadata, never the cookie.
- If `pixiv_id` is supplied, fetches that Pixiv work as the live validation action.

Request:

```json
{
  "pixiv_id": "144920810"
}
```

Response:

```json
{
  "data": {
    "configured": true,
    "status": "ok",
    "pixiv_id": "144920810",
    "title": "title"
  }
}
```

### `POST /api/settings/test/deepseek`

Tests DeepSeek configuration without exposing the API key.

Implemented in Phase 6A. Uses the configured `deepseek_api_key`, `deepseek_base_url`, and `deepseek_model`. Phase 6B tightened the prompt guidance so Japanese Pixiv tags are preferred with English fallback, and user-selected R18 policy is not overridden by model output.

Default model: `deepseek-v4-flash`.

Response:

```json
{
  "data": {
    "configured": true,
    "status": "ok",
    "model": "deepseek-v4-flash"
  }
}
```

# Requirements Specification

Source PRD: `docs/product_requirements.md`

## V1 Product Goal

Build a local-first personal Pixiv image downloader and smart retrieval platform. The platform downloads Pixiv images through user-provided credentials, indexes them in SQLite, stores files locally, and provides an anime-image-oriented gallery with smart search, task visibility, and theme customization.

## Scope Boundary

V1 includes local usage only. It does not include account systems, cloud sync, Docker packaging, Baidu Cloud integration, or automatic Pixiv cookie refresh.

## Requirement List

### Pixiv Download

| ID | Requirement | Acceptance Criteria |
| --- | --- | --- |
| `REQ-DL-001` | Download a single Pixiv work by work ID. | User submits a Pixiv ID, backend creates a task, downloads original image files when available, persists metadata and local paths. |
| `REQ-DL-002` | Download bookmarked works. | User selects quantity and category/R18 policy, backend fetches bookmarks and queues downloads up to the requested limit. |
| `REQ-DL-003` | Download works by author UID. | User submits author UID and limit, backend downloads matching author works and records `author_uid`. |
| `REQ-DL-004` | Fetch and display Pixiv daily Top10. | Homepage can refresh Top10, display carousel items, and optionally download selected/top items. |
| `REQ-DL-005` | Random Surprise download. | User can trigger a random Pixiv work discovery/download path with task visibility. |
| `REQ-DL-006` | Deduplicate images by Pixiv work/page identity. | Repeated downloads do not create duplicate image rows or overwrite unrelated files. |
| `REQ-DL-007` | Manual Pixiv cookie input. | V1 accepts user-provided `PHPSESSID`; no automatic refresh is required. |

### Smart Retrieval

| ID | Requirement | Acceptance Criteria |
| --- | --- | --- |
| `REQ-AI-001` | Parse natural language into Pixiv search intent. | DeepSeek returns structured tags, optional negative tags, recommended count, and R18/NSFW intent. |
| `REQ-AI-002` | Execute tag-based retrieval from AI output. | Backend searches Pixiv using generated tags and creates a smart retrieval task. |
| `REQ-AI-003` | Preserve smart retrieval provenance. | Store original user prompt, normalized AI output, and final execution parameters. |
| `REQ-AI-004` | Allow user quantity override. | User can override AI recommended count before task submission. |
| `REQ-AI-005` | Handle AI failure gracefully. | Failed parsing returns actionable error state without creating ambiguous download records. |

### Image Management

| ID | Requirement | Acceptance Criteria |
| --- | --- | --- |
| `REQ-IMG-001` | Store image metadata in SQLite. | Records include Pixiv ID, page index, author UID, tags, category, source, path, and timestamps. |
| `REQ-IMG-002` | Support gallery filtering. | Gallery filters by tag, category, author, source, date, and R18 visibility. |
| `REQ-IMG-003` | Support waterfall display with lazy loading. | Gallery loads image batches progressively and does not block initial render on entire library. |
| `REQ-IMG-004` | Support click-to-zoom preview. | User can open an image preview with metadata and navigation controls. |
| `REQ-IMG-005` | Support map-based or heatmap-style search. | System stores optional visual coordinates for tag/style exploration. |
| `REQ-IMG-006` | Maintain category and tag editability. | User can adjust categories/tags locally without re-downloading. |
| `REQ-IMG-007` | Support deleting local images from Gallery. | User can delete one or more selected images, removing local files and keeping SQLite indexes consistent. |

### Async Tasks

| ID | Requirement | Acceptance Criteria |
| --- | --- | --- |
| `REQ-TASK-001` | All download operations run as async tasks. | Frontend receives a `task_id`; task processing happens in background. |
| `REQ-TASK-002` | Frontend can poll task status. | API returns status, progress, logs, created/finished timestamps, and per-item failures. |
| `REQ-TASK-003` | Task state transitions are explicit. | Tasks move through `pending`, `running`, terminal states, and never silently disappear. |
| `REQ-TASK-004` | Failed tasks preserve diagnostics. | Error code/message and logs are stored for user review. |
| `REQ-TASK-005` | Task progress is monotonic. | Completed count does not decrease; terminal tasks no longer mutate progress except final metadata corrections. |

### Settings

| ID | Requirement | Acceptance Criteria |
| --- | --- | --- |
| `REQ-CFG-001` | Configure Pixiv cookie. | User can save, update, and test PHPSESSID. |
| `REQ-CFG-002` | Configure download path. | User can choose local base path; default is repository `output/` via `project:output`. |
| `REQ-CFG-003` | Configure DeepSeek settings. | User can save API key, base URL, and model; defaults are `https://api.deepseek.com` and `deepseek-v4-flash`. |
| `REQ-CFG-004` | Configure request limits. | Frontend and backend enforce max quantity per request. |
| `REQ-CFG-005` | Configure R18/NSFW policy. | User can choose visibility/download policy, and UI reflects it consistently. |
| `REQ-CFG-006` | Configure visual theme. | User can switch between Cyan Studio and Sakura Light. |
| `REQ-CFG-007` | Configure default batch quantity. | Batch and smart retrieval requests use a configurable default count when the user does not provide one. |

### Frontend UX

| ID | Requirement | Acceptance Criteria |
| --- | --- | --- |
| `REQ-UI-001` | Provide homepage. | Shows Daily Top10 carousel, Random Surprise, category entry, and recent task highlights. |
| `REQ-UI-002` | Provide download page. | Supports single ID, bookmarks, author, Top10, random, and smart retrieval entry points. |
| `REQ-UI-003` | Provide task panel. | Shows queue, task status, progress bars, logs, filters, and terminal states. |
| `REQ-UI-004` | Provide settings page. | Contains cookie, download path, DeepSeek, limits, R18 policy, and theme controls. |
| `REQ-UI-005` | Provide gallery page. | Waterfall grid, filters, map search, click-to-zoom, and lazy loading. |
| `REQ-UI-006` | Preserve state across navigation. | Current filters, selected theme, and visible task state should survive normal navigation. |

### Visual Theme

| ID | Requirement | Acceptance Criteria |
| --- | --- | --- |
| `REQ-THEME-001` | Use an anime/Pixiv-compatible visual language. | UI feels image-first, polished, bright, and expressive without reducing gallery density. |
| `REQ-THEME-002` | Provide Sakura Light theme demo. | Sakura Light preview uses real layout fragments, not only swatches. |
| `REQ-THEME-003` | Avoid one-note color systems. | Themes combine neutrals, accent colors, status colors, and image-friendly surfaces. |
| `REQ-THEME-004` | Keep operational pages efficient. | Download, task, and settings pages remain scannable and tool-like rather than landing-page-like. |
| `REQ-THEME-005` | Protect R18 content by design. | R18 thumbnails can be hidden, blurred, or gated according to settings. |

### Security and Privacy

| ID | Requirement | Acceptance Criteria |
| --- | --- | --- |
| `REQ-SEC-001` | Keep credentials local. | Pixiv cookie and DeepSeek key are stored locally and never sent to frontend after save except masked status. |
| `REQ-SEC-002` | Avoid leaking local paths unnecessarily. | API can expose display paths/IDs; full local paths appear only where useful for settings/debug views. |
| `REQ-SEC-003` | Treat R18 state as a first-class visibility state. | R18 category is not inferred only from tags; it is persisted and filtered explicitly. |

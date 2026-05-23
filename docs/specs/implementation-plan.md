# Implementation Plan

Strategy: build the core downloader first, behind testable interfaces, then wrap it with tasks/API/frontend.

## Milestone 0: Spec Lock For Downloader

Status: completed

Deliverables:

- `architecture.md`
- `pixiv-client.md`
- `file-storage.md`
- `error-catalog.md`
- `testing-strategy.md`

Gate:

- User confirms live test credentials can be supplied later through environment variables.

## Milestone 1: Backend Core Scaffold

Requirements: `REQ-DL-001`, `REQ-DL-006`, `REQ-IMG-001`, `REQ-TASK-004`

Status: completed

Deliverables:

- `src/backend/Cargo.toml`
- Typed `AppError`
- Domain enums and DTOs
- `PixivClient` trait
- mock Pixiv client
- file storage planner
- temp-file-safe writer

Tests:

- Filename/path planning.
- Error code formatting.
- Mock image byte write.
- R18 policy skip logic.

Current evidence:

- `./tests/run_local.sh`
- `6 passed; 0 failed`

## Milestone 2: SQLite and Dedupe

Requirements: `REQ-DL-006`, `REQ-IMG-001`, `REQ-TASK-002`

Status: completed

Deliverables:

- Initial migrations.
- Image repository.
- Task/task item/log repository.
- Settings repository.

Tests:

- Migration creates tables.
- `pixiv_id + page_index` uniqueness.
- Duplicate skip.
- Missing file repair state.

Implementation flow:

1. Add SQLite dependency and DB module. Done.
2. Create `migrations/0001_init.sql`. Done.
3. Implement migration runner. Done.
4. Add migration tests. Done.
5. Implement image/tag/source repositories. Done.
6. Add temporary-DB repository tests. Done.
7. Add settings repository skeleton with masked secret DTO behavior. Done.
8. Wire downloader dedupe to DB. Done in Phase 2B.

Current evidence:

- `./tests/run_local.sh`
- `82 backend unit tests passed; Phase 2A checks passed; Phase 2C checks passed; backend SQLite integration checks passed; backend API smoke checks passed; Phase 3B queue checks passed; Phase 4B data API checks passed; Phase 4C configured download checks passed; Phase 4D gallery file API checks passed; Phase 4E gallery delete checks passed; Phase 5A author batch checks passed; Phase 5B bookmark batch checks passed; Phase 6A smart parse checks passed; Phase 6B smart download checks passed; frontend scaffold checks passed; 0 failed`

## Milestone 3: Single Work Downloader

Requirements: `REQ-DL-001`, `REQ-DL-006`, `REQ-SEC-001`

Status: completed for DB-aware persistence and task lifecycle wrapper

Deliverables:

- `download_single` orchestrator.
- Metadata fetch through `PixivClient`.
- Original image download.
- File write.
- SQLite image insert. Done through DB-aware downloader.
- Source history insert. Done through DB-aware downloader.

Tests:

- Mock single work success.
- Multi-page work success.
- Duplicate skip.
- R18 excluded skip.
- Network failure.
- Filesystem failure.

## Milestone 4: Live Pixiv Smoke

Requirements: `REQ-DL-001`, `REQ-DL-007`

Status: passed for single work `144920810` on 2026-05-21.

Deliverables:

- `tests/live` smoke notes or script.
- Env-based credentials.
- Limit-one real download test.

Required user input:

- `PIXIV_PHPSESSID`
- `PIXIV_TEST_WORK_ID`

Gate:

- User explicitly allows live network download.

## Milestone 5: Task Queue and API Wrapper

Requirements: `REQ-TASK-001` through `REQ-TASK-005`, `REQ-UI-003`

Status: Phase 3A minimal API wrapper and Phase 3B background Tokio queue complete

Deliverables:

- Axum route for single download. Done: `POST /api/download/single`.
- Task creation. Done for repository/worker wrapper.
- Tokio worker. Done for Phase 3B.
- Polling endpoint. Done: `GET /api/tasks/{task_id}`.
- Task logs. Done for repository/worker wrapper.
- Final cleanup check. Done for Phase 3B; generated candidate list and did not delete anything.

Tests:

- API returns `202` and `task_id`. Done: `req_dl_001_req_task_001_post_single_download_enqueues_and_returns_task_id`.
- Worker updates states. Done in task lifecycle tests.
- Failed task preserves diagnostics. Done in task lifecycle tests.
- Task polling returns items/logs. Done: `req_task_002_req_task_004_get_task_returns_items_and_logs`.
- Enqueue returns before work must finish. Done: `req_dl_001_req_task_001_post_single_download_enqueues_and_returns_task_id`.
- Polling can observe queued/running/terminal task snapshots. Done: `req_task_002_req_task_004_get_task_returns_items_and_logs`, `req_task_004_queued_single_download_preserves_failure_diagnostics`.

## Milestone 6: Frontend Data API Wiring

Requirements: `REQ-IMG-002`, `REQ-IMG-003`, `REQ-CFG-001`, `REQ-SEC-001`, `REQ-TASK-002`, `REQ-UI-003`, `REQ-UI-004`, `REQ-UI-005`

Status: Phase 4B minimum viable slice complete

Deliverables:

- Task list endpoint. Done: `GET /api/tasks`.
- Gallery metadata list/detail endpoints. Done: `GET /api/images`, `GET /api/images/{image_id}`.
- Settings public list/save endpoints. Done: `GET /api/settings`, `PUT /api/settings/{key}`.
- Gallery page real metadata integration. Done.
- Settings page real public settings integration. Done.
- Tasks page recent task list integration. Done.

Tests:

- Gallery repository filter/cursor test. Done: `req_img_002_req_img_003_repository_lists_images_with_filters_and_cursor`.
- Gallery API metadata test. Done: `req_img_002_req_ui_005_get_images_returns_gallery_metadata`.
- Settings save/mask repository and API tests. Done: `req_cfg_001_req_sec_001_settings_repository_saves_known_values_and_masks_secret`, `req_cfg_001_req_sec_001_settings_api_lists_and_saves_masked_values`.
- Task list repository/API tests. Done: `req_task_002_repository_lists_tasks_with_filters_and_cursor`, `req_task_002_req_ui_003_get_tasks_returns_task_list`.
- Phase script. Done: `tests/stage/phase4b_data_api.sh`.

## Milestone 7: Frontend-Configured Single Download

Requirements: `REQ-DL-001`, `REQ-DL-007`, `REQ-CFG-001`, `REQ-CFG-002`, `REQ-SEC-001`, `REQ-UI-002`, `REQ-UI-004`

Status: complete

Goal:

- Let a user configure Pixiv credential and download directory from the frontend, submit a single-work download, and verify the file lands in the configured directory.

Deliverables:

- Settings-backed Pixiv cookie resolution for API/worker runtime. Done.
- Settings-backed `download_base_path` resolution for worker storage root. Done.
- Path validation/normalization, including `~` expansion and directory creation. Done.
- `POST /api/settings/test/pixiv` for credential validation without exposing the cookie. Done.
- Frontend copy/control polish for Settings and Download so the manual test flow is clear. Done for Settings Pixiv test control.

Tests:

- Settings-backed downloader uses configured download root with mock Pixiv data. Done: `req_dl_001_req_cfg_002_single_download_uses_settings_cookie_and_download_root`.
- Settings-backed Pixiv credential is masked in public API responses. Done.
- Missing credential still fails with a stable error. Done: `req_dl_007_settings_backed_pixiv_cookie_is_required_before_enqueue`.
- Unsafe/empty download roots are rejected or normalized deterministically. Done.
- `./tests/run_local.sh` remains green. Done.

## Milestone 8: Gallery File Preview Slice

Requirements: `REQ-IMG-004`, `REQ-SEC-002`, `REQ-UI-005`

Status: complete

Goal:

- Make downloaded files visible in Gallery without exposing local filesystem paths.

Deliverables:

- Image file lookup helper in repository/query layer. Done.
- `GET /api/images/{image_id}/file` secure byte response. Done.
- Minimum `preview_url` / `thumbnail_url` values in gallery DTOs. Done.
- Gallery cards render real downloaded images. Done.
- Image detail view uses metadata plus preview. Done.

Tests:

- Existing file returns bytes and image content type. Done.
- Unknown image id returns `404`. Done.
- Missing local file returns `404` or a stable file-missing error. Done.
- Unsafe stored path is rejected. Done.
- JSON metadata responses do not expose local paths. Done.
- Phase 4D stage script is included in `./tests/run_local.sh`. Done.

## Milestone 9: Batch Download Modes

Requirements: `REQ-DL-002`, `REQ-DL-003`, `REQ-DL-004`, `REQ-DL-005`

Status: Phase 5A and Phase 5B complete; Phase 5C planned next

Deliverables:

- Phase 5A author download through `POST /api/downloads/author`. Done.
- Shared sequential batch worker behavior with multiple task items. Done for author and bookmark batches.
- Partial-failure accounting with `completed_with_errors`. Done for author and bookmark batches.
- Phase 5B bookmark download through `POST /api/downloads/bookmarks`. Done.
- Phase 5C daily Top10 refresh/download.
- Phase 5D random surprise.

Tests:

- Mock author discovery and batch processing. Done.
- Mock bookmark discovery and batch processing. Done.
- Limit enforcement. Done for author and bookmark batches.
- Partial failure produces `completed_with_errors`. Done for author and bookmark batches.
- Duplicate skip and source history remain DB-aware.
- R18 policy respected.
- Live batch checks are opt-in with tiny limits.

## Milestone 10: Smart Retrieval

Requirements: `REQ-AI-001` through `REQ-AI-005`

Status: Phase 6B tag search/download slice complete

Deliverables:

- DeepSeek parser. Done in Phase 6A.
- Smart download task. Done in Phase 6B.
- Provenance persistence. Done in Phase 6B.
- Pixiv tag search discovery. Done in Phase 6B.

Tests:

- Structured AI output parsing. Done.
- User count override. Done for parse validation.
- Smart tag search parser, smart task worker, provenance, missing-cookie, and count cap validation. Done.
- AI failure state.

## Milestone 11: Frontend Integration

Requirements: `REQ-UI-001` through `REQ-UI-006`, `REQ-THEME-001` through `REQ-THEME-005`

Status: scaffold, Phase 4B data integration, Phase 4C settings-backed download flow, Phase 4D gallery file previews, Phase 5A author batch form, and Phase 5B bookmark batch form complete; remaining batch/smart flows pending

Deliverables:

- Next.js scaffold. Done.
- Cyan Studio theme. Done.
- Download center. Done for single-work API, bookmark batch API, and author batch API.
- Task panel. Done for task-id polling and recent task list.
- Gallery MVP. Done for metadata list and secure original-file preview; generated thumbnail cache pending.
- Settings page. Done for public list/save; connection test APIs pending.

Tests:

- Frontend typecheck. Done: `tests/stage/frontend_scaffold.sh`.
- Production build. Done: `tests/stage/frontend_scaffold.sh`.
- Route rendering. Covered by build for scaffold routes.
- Task polling UI. Basic task-id polling implemented.
- R18 visibility states.
- Gallery lazy loading.

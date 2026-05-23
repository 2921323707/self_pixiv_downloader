# Project Progress

Last updated: 2026-05-23

## Current Anchor

The project is anchored on a downloader-first implementation path.

Current phase: **Phase 7B - UI Layout / Interaction Polish** first pass complete.

The core Pixiv single-work download path is proven with mock tests, one live Pixiv smoke test, DB-aware local indexing, task-state persistence, deterministic stage/integration test scripts, a thin Axum API wrapper, an in-process Tokio background queue/worker, a Next.js frontend scaffold, minimal gallery/settings/task-list data APIs, settings-backed Pixiv cookie/download-root resolution for frontend-initiated downloads, secure local image file serving for Gallery previews, Gallery hard-delete file/index cleanup, author batch download, bookmark batch download, DeepSeek-backed smart prompt parsing, smart tag-search batch download, a Home dashboard backed by real task/image/settings APIs, and a first UI polish pass across Home/Download/Tasks/Gallery/Settings.

Manual browser anchor on 2026-05-22: Settings-saved Pixiv credential plus a single Pixiv ID submitted from the frontend successfully downloaded a work, completed the task flow, and wrote the file locally. User also manually validated Author Batch and Bookmarks Batch through the frontend.

Manual browser anchor on 2026-05-23: user confirmed the Phase 6B Smart Retrieval flow has no obvious current issues after the Parse -> editable tags -> enqueue smart download fix.

Manual browser anchor on 2026-05-23: Home Dashboard loaded through the local frontend/backend with recent tasks, recent image previews, masked configuration status, and no browser console errors.

Implementation anchor on 2026-05-23: Phase 7B UI polish changed Download into a tabbed workbench, Gallery image details into a right-side drawer, Tasks details into a centered modal with the recent list defaulting to 10 rows plus expand-more, Settings into categorized panels, and Home into a less cramped workbench with a recent normal image banner. No new backend APIs were required.

## Current Reality Check

Backend status: the downloader-first backend core is solid for the current vertical slice, but the full product backend is not complete yet.

Completed backend slice:

- Single Pixiv work download core.
- SQLite persistence for images, tags, sources, tasks, task items, and task logs.
- DB-aware dedupe / missing-file repair / existing-file indexing.
- Thin Axum API for `POST /api/download/single` and `GET /api/tasks/{task_id}`.
- In-process Tokio background queue for single-download tasks.
- Task list API: `GET /api/tasks` with status/type/limit/cursor.
- Gallery metadata APIs: `GET /api/images` and `GET /api/images/{image_id}` with filters and cursor pagination.
- Gallery delete APIs: `DELETE /api/images/{image_id}` and `POST /api/images/delete-batch`.
- Settings APIs: `GET /api/settings` and `PUT /api/settings/{key}` with repository-level allowlist validation and secret masking.
- Settings-backed runtime resolution for single downloads: `pixiv_cookie` and `download_base_path`.
- Default download root is repository `output/` via `project:output` when no explicit path is configured.
- Pixiv connection test API: `POST /api/settings/test/pixiv`.
- DeepSeek connection test API: `POST /api/settings/test/deepseek`.
- Smart parse API: `POST /api/smart/parse`.
- Author batch API: `POST /api/downloads/author`.
- Author work discovery through the Pixiv client abstraction.
- Bookmark batch API: `POST /api/downloads/bookmarks`.
- Current-user bookmark discovery through the Pixiv client abstraction.
- Sequential multi-item worker dispatch with per-item diagnostics and `completed_with_errors`.
- Batch default count setting: `default_batch_count`, default `20`, capped by `max_request_count`.

Backend still pending:

- Generated thumbnail cache APIs.
- Top10 / random batch modes.
- Richer image preview, edit, and map APIs.

Frontend status: the current frontend is a working scaffold, not a full product UI.

- Real integration exists for single-download submission, bookmark batch submission, author batch submission, Smart Retrieval parse preview and smart batch enqueue, settings-backed Pixiv cookie/download directory, DeepSeek settings/test, task-id polling, task list, gallery metadata, Gallery delete, and public settings list/save.
- Home now shows a real dashboard by reusing `GET /api/tasks`, `GET /api/images`, and `GET /api/settings`, including a recent normal image banner.
- Top10/Random download sections are still pending.
- Gallery now renders real downloaded image previews through a secure file endpoint, supports multi-select hard delete, and opens image detail in a right-side drawer.
- Download uses a balanced tabbed workbench for Single / Author / Bookmarks / Smart instead of a large-left plus stacked-right layout.
- Tasks opens live task progress/items/logs in a centered modal and keeps Recent Tasks compact by default.
- Settings groups existing public settings into General, Appearance, Pixiv, DeepSeek, and Storage panels while keeping secrets masked.

## Current Task Status

The Phase 7B UI Layout / Interaction Polish first pass is complete.

| Task | Status | Evidence |
| --- | --- | --- |
| Create Next.js frontend scaffold | Done | `src/frontend/package.json`, `src/frontend/app` |
| Add Cyan Studio app shell | Done | `src/frontend/components/AppShell.tsx`, `src/frontend/app/globals.css` |
| Add placeholder routes | Done | `/`, `/download`, `/tasks`, `/gallery`, `/settings` |
| Wire single download API | Done | `src/frontend/app/download/page.tsx`, `src/frontend/lib/api.ts` |
| Wire task polling by task id | Done | `src/frontend/app/tasks/page.tsx` |
| Add frontend quality script | Done | `tests/stage/frontend_scaffold.sh` |
| Run `./tests/run_local.sh` | Done | Backend checks plus frontend typecheck/build passed |
| Add gallery metadata API | Done | `src/backend/src/images/mod.rs`, `src/backend/src/api.rs` |
| Add settings list/save API | Done | `src/backend/src/settings/mod.rs`, `src/backend/src/api.rs` |
| Replace Home placeholders with real dashboard | Done | `src/frontend/app/page.tsx`, `src/frontend/app/globals.css` |
| Reuse task/image/settings APIs on Home | Done | `src/frontend/app/page.tsx`, `src/frontend/lib/api.ts` |
| Add Home dashboard frontend gate | Done | `tests/stage/frontend_scaffold.sh` |
| Add Home normal image banner | Done | `src/frontend/app/page.tsx`, `src/frontend/app/globals.css` |
| Convert Download to tabbed workbench | Done | `src/frontend/app/download/page.tsx`, `src/frontend/app/globals.css` |
| Convert Gallery detail to right drawer | Done | `src/frontend/app/gallery/page.tsx`, `src/frontend/app/globals.css` |
| Convert Tasks detail to modal and default recent list to 10 | Done | `src/frontend/app/tasks/page.tsx`, `src/frontend/app/globals.css` |
| Convert Settings to categorized panels | Done | `src/frontend/app/settings/page.tsx`, `src/frontend/app/globals.css` |
| Add Phase 7B frontend scaffold anchors | Done | `tests/stage/frontend_scaffold.sh` |
| Add task list API | Done | `src/backend/src/tasks/mod.rs`, `src/backend/src/api.rs` |
| Wire Gallery / Settings / Tasks to real data | Done | `src/frontend/app/gallery/page.tsx`, `src/frontend/app/settings/page.tsx`, `src/frontend/app/tasks/page.tsx` |
| Add Phase 4B deterministic script | Done | `tests/stage/phase4b_data_api.sh` |
| Add settings-backed download runtime | Done | `src/backend/src/api.rs` |
| Add Pixiv connection test API | Done | `POST /api/settings/test/pixiv` |
| Add Settings Pixiv test control | Done | `src/frontend/app/settings/page.tsx` |
| Add Phase 4C deterministic script | Done | `tests/stage/phase4c_configured_download.sh` |
| Complete manual browser single-download validation | Done | User manually downloaded by Pixiv ID through the frontend on 2026-05-22 |
| Add secure gallery file endpoint | Done | `GET /api/images/{image_id}/file`, `src/backend/src/api.rs` |
| Add image file resolution helper | Done | `resolve_image_file`, `src/backend/src/images/mod.rs` |
| Wire Gallery previews to real image bytes | Done | `src/frontend/app/gallery/page.tsx` |
| Add Phase 4D deterministic script | Done | `tests/stage/phase4d_gallery_file_api.sh` |
| Add default batch count setting | Done | `src/backend/src/settings/mod.rs`, `src/backend/migrations/0001_init.sql` |
| Add author discovery to Pixiv client | Done | `fetch_author_works`, `src/backend/src/pixiv/mod.rs` |
| Add author batch task lifecycle | Done | `create_author_download_task`, `execute_author_download_task`, `src/backend/src/tasks/mod.rs` |
| Add author batch API | Done | `POST /api/downloads/author`, `src/backend/src/api.rs` |
| Wire Download Author form | Done | `src/frontend/app/download/page.tsx`, `src/frontend/lib/api.ts` |
| Add Phase 5A deterministic script | Done | `tests/stage/phase5a_author_batch.sh` |
| Add bookmark discovery to Pixiv client | Done | `fetch_bookmarks`, `src/backend/src/pixiv/mod.rs` |
| Add bookmark batch task lifecycle | Done | `create_bookmark_download_task`, `execute_bookmark_download_task`, `src/backend/src/tasks/mod.rs` |
| Add bookmark batch API | Done | `POST /api/downloads/bookmarks`, `src/backend/src/api.rs` |
| Wire Download Bookmarks form | Done | `src/frontend/app/download/page.tsx`, `src/frontend/lib/api.ts` |
| Add Phase 5B deterministic script | Done | `tests/stage/phase5b_bookmark_batch.sh` |
| Add Gallery hard-delete repository/API | Done | `delete_image_file_and_index`, `DELETE /api/images/{image_id}`, `POST /api/images/delete-batch` |
| Wire Gallery multi-select delete | Done | `src/frontend/app/gallery/page.tsx`, `src/frontend/lib/api.ts` |
| Add Phase 4E deterministic script | Done | `tests/stage/phase4e_gallery_delete.sh` |
| Add DeepSeek model setting default | Done | `deepseek_model = deepseek-v4-flash`, `src/backend/src/settings/mod.rs` |
| Add DeepSeek HTTP client and parser | Done | `src/backend/src/ai.rs` |
| Add Smart Parse API | Done | `POST /api/smart/parse`, `src/backend/src/api.rs` |
| Add DeepSeek connection test API | Done | `POST /api/settings/test/deepseek`, `src/backend/src/api.rs` |
| Wire Settings DeepSeek test/model | Done | `src/frontend/app/settings/page.tsx`, `src/frontend/lib/api.ts` |
| Wire Download Smart parse preview | Done | `src/frontend/app/download/page.tsx`, `src/frontend/lib/api.ts` |
| Add Phase 6A deterministic script | Done | `tests/stage/phase6a_smart_parse.sh` |
| Improve Smart Parse tag policy | Done | Japanese Pixiv tags preferred, English fallback, user-selected R18 policy wins |
| Add Pixiv smart tag search | Done | `search_works_by_tags`, `src/backend/src/pixiv/mod.rs` |
| Add smart batch task lifecycle | Done | `create_smart_download_task`, `execute_smart_download_task`, `src/backend/src/tasks/mod.rs` |
| Persist smart retrieval provenance | Done | `smart_retrievals`, prompt/model/tags/count/r18 stored when task is created |
| Add Smart Download API | Done | `POST /api/smart/download`, `src/backend/src/api.rs` |
| Wire Download Smart enqueue | Done | `src/frontend/app/download/page.tsx`, `src/frontend/lib/api.ts`; parsed tags are editable before enqueue |
| Add Phase 6B deterministic script | Done | `tests/stage/phase6b_smart_download.sh` |

## Completed

| Area | Status | Evidence |
| --- | --- | --- |
| PRD understanding | Done | `docs/product_requirements.md` reviewed and decomposed into specs |
| Spec structure | Done | `docs/specs/` contains requirements, domain, DB, API, frontend, theme, task flow, testing, traceability |
| Theme direction | Done | Demo B selected as `cyan-studio`; `demo_B.html` retained as reference |
| Downloader-first architecture | Done | `docs/specs/architecture.md` |
| Pixiv client strategy | Done | `docs/specs/pixiv-client.md` |
| File layout strategy | Done | `docs/specs/file-storage.md` |
| Error catalog | Done | `docs/specs/error-catalog.md` |
| Rust backend core scaffold | Done | `src/backend` |
| Mock single-work downloader | Done | `req_dl_001_downloads_single_work_with_mock_pixiv` |
| Live Pixiv single-work smoke | Done | `tests/live/README.md`, work `144920810` |
| Test script baseline | Done | `tests/run_local.sh`, `tests/unit/backend_unit.sh`, `tests/e2e/live_single_download.sh` |
| Phase 2A test script | Done | `tests/stage/phase2a_repository.sh` |
| Cleanup | Done | Removed `.DS_Store`, unselected demos, temporary live download artifact |
| SQLite dependency | Done | `rusqlite` added in `src/backend/Cargo.toml` |
| Initial migration | Done | `src/backend/migrations/0001_init.sql` |
| DB migration runner | Done | `src/backend/src/db/mod.rs` |
| Image repository | Done | `src/backend/src/images/mod.rs` |
| Settings repository | Done | `src/backend/src/settings/mod.rs` |
| DB-aware downloader | Done | `download_single_with_db`, Phase 2B downloader tests |
| Task persistence | Done | `src/backend/src/tasks/mod.rs`, Phase 2C task tests |
| Test script expansion | Done | `phase2c_tasks.sh`, `backend_sqlite.sh`, expanded `run_local.sh` |
| Axum API wrapper | Done | `src/backend/src/api.rs`, `src/backend/src/bin/server.rs`, `tests/smoke/backend_api.sh` |
| Background task queue | Done | `src/backend/src/api.rs`, `src/backend/src/tasks/mod.rs`, `tests/stage/phase3b_queue.sh` |
| Frontend scaffold | Done | `src/frontend`, `tests/stage/frontend_scaffold.sh` |
| Phase 4B data API wiring | Done | `tests/stage/phase4b_data_api.sh`, Gallery/Settings/Tasks frontend pages |
| Phase 4C configured single download | Done | `tests/stage/phase4c_configured_download.sh`, settings-backed worker runtime |
| Phase 4D gallery file preview | Done | `tests/stage/phase4d_gallery_file_api.sh`, Gallery preview UI |
| Phase 4E gallery delete | Done | `tests/stage/phase4e_gallery_delete.sh`, Gallery selection delete UI |
| Phase 5A author batch download | Done | `tests/stage/phase5a_author_batch.sh`, `POST /api/downloads/author`, Download Author form |
| Phase 5B bookmark batch download | Done | `tests/stage/phase5b_bookmark_batch.sh`, `POST /api/downloads/bookmarks`, Download Bookmarks form |
| Phase 6A smart parse | Done | `tests/stage/phase6a_smart_parse.sh`, `POST /api/smart/parse`, Download Smart Retrieval form |
| Phase 6B smart download | Done | `tests/stage/phase6b_smart_download.sh`, `POST /api/smart/download`, smart provenance |
| Phase 7A Home dashboard | Done | `src/frontend/app/page.tsx`, real task/image/settings APIs |
| Phase 7B UI polish | Done | `src/frontend/app/{page.tsx,download/page.tsx,gallery/page.tsx,tasks/page.tsx,settings/page.tsx}`, `tests/stage/frontend_scaffold.sh` |
| Document map | Done | `docs/DOCUMENT_MAP.md` |
| Root README | Done | `README.md` |

## Current Verification

Run:

```text
./tests/run_local.sh
```

Current result:

```text
./tests/run_local.sh
82 backend unit tests passed; Phase 2A checks passed; Phase 2C checks passed; backend SQLite integration checks passed; backend API smoke checks passed; Phase 3B queue checks passed; Phase 4B data API checks passed; Phase 4C configured download checks passed; Phase 4D gallery file API checks passed; Phase 4E gallery delete checks passed; Phase 5A author batch checks passed; Phase 5B bookmark batch checks passed; Phase 6A smart parse checks passed; Phase 6B smart download checks passed; frontend scaffold checks passed; 0 failed
```

Latest focused frontend gate:

```text
./tests/stage/frontend_scaffold.sh
frontend scaffold checks passed, including Phase 7B UI polish anchors, TypeScript check, and production build
```

Live E2E is opt-in:

```text
PIXIV_PHPSESSID=... ./tests/e2e/live_single_download.sh
```

Current live E2E script status:

- Uses `download_single_with_db`.
- Wraps the live single-work path in `run_single_download_task`.
- Defaults to a unique temporary download directory and SQLite DB.
- Verifies file existence, non-empty bytes, task completion, task items/logs, image DB row, source history, and second-run DB duplicate skip.
- Requires `PIXIV_PHPSESSID` at runtime; credentials must not be written to repository files.

Manual frontend validation:

- 2026-05-22: user confirmed browser flow can download a Pixiv work by ID.
- This validates the frontend Settings -> Download -> Tasks -> local file path for the current single-work vertical slice.
- 2026-05-22: browser check confirmed Gallery renders local image previews and an image detail panel through the secure file endpoint with no console errors.
- 2026-05-22: deterministic Phase 5A checks confirmed author batch enqueue, default count resolution, max limit rejection, missing-cookie rejection, multi-item completion, and partial failure diagnostics. Live author batch remains opt-in.
- 2026-05-22: user confirmed Author Batch works manually through the frontend.
- 2026-05-22: deterministic Phase 5B checks confirmed bookmark batch enqueue, default count resolution, max limit rejection, missing-cookie rejection, multi-item completion, and partial failure diagnostics. Live bookmark batch remains opt-in.
- 2026-05-22: user confirmed Bookmarks Batch works manually through the frontend.
- 2026-05-22: deterministic Phase 4E checks confirmed Gallery delete removes local files and SQLite indexes, handles already-missing files, rejects unsafe paths, and returns batch per-item diagnostics.
- 2026-05-22: deterministic Phase 6A checks confirmed DeepSeek settings/model defaults, masked-key connection tests, smart prompt parsing, missing-key errors, count limit validation, and frontend typecheck/build.
- 2026-05-23: deterministic Phase 6B checks confirmed Pixiv smart tag search parsing, smart task enqueue, worker execution, source history, `smart_retrievals` provenance, missing-cookie rejection, count limit validation, and frontend typecheck.
- 2026-05-23: user manually confirmed the Smart Retrieval frontend flow is currently acceptable after tag editing and smart enqueue were added.

## Next Phase Todo

### Phase 2A: SQLite Migrations and Repositories

Requirements: `REQ-IMG-001`, `REQ-DL-006`, `REQ-TASK-002`, `REQ-SEC-001`

Goal: make downloaded image metadata persistent and dedupe by database identity.

Status: complete.

Tasks:

1. Add SQLite dependency and migration runner. Done.
2. Create migration `0001_init.sql`. Done.
3. Add migration tests using temporary in-memory SQLite. Done.
4. Implement `images` repository. Done.
5. Implement `image_tags` repository behavior. Done.
6. Implement `image_sources` repository behavior. Done, including task-linked source history when downloads run through the task wrapper.
7. Add settings repository skeleton with masked secret DTO behavior. Done.
8. Add repository integration tests using temporary SQLite DB. Done.

Acceptance:

- Migration creates all V1 tables.
- `pixiv_id + page_index` uniqueness is enforced.
- Downloaded metadata can be inserted and queried. Done.
- Tags can be inserted and queried as arrays. Done.
- Source history can record `single`. Done.
- Secret settings are never returned unmasked.

### Phase 2B: DB-Aware Downloader

Requirements: `REQ-DL-001`, `REQ-DL-006`, `REQ-IMG-001`

Goal: upgrade current file-only downloader into DB + filesystem downloader.

Status: complete.

Tasks:

1. Add `DownloadRepositoryContext` or equivalent orchestration context. Done.
2. Check DB dedupe before downloading bytes. Done.
3. Repair DB row when DB exists but file is missing. Done.
4. Index existing file when file exists but DB row is missing. Done.
5. Insert image metadata after successful file write. Done.
6. Insert tags and source history. Done.

Acceptance:

- First download writes file and DB rows. Done: `req_dl_001_req_img_001_db_aware_first_download_indexes_file_tags_and_source`.
- DB duplicate skip returns duplicate/skip from DB state and avoids byte download. Done: `req_dl_006_db_duplicate_skip_avoids_image_download_and_records_source`.
- Missing-file repair path is covered by test. Done: `req_dl_006_missing_file_repair_redownloads_and_refreshes_index`.
- Existing-file indexing path is covered by test. Done: `req_dl_006_existing_file_indexing_inserts_db_without_downloading_bytes`.

### Phase 2C: Task State Persistence

Requirements: `REQ-TASK-001` through `REQ-TASK-005`

Goal: every download becomes traceable as a task even before the frontend exists.

Status: complete.

Tasks:

1. Implement `tasks` repository. Done.
2. Implement `task_logs` repository. Done.
3. Implement `task_items` repository. Done.
4. Add task state transition helpers. Done.
5. Wrap single download in task lifecycle: `pending -> running -> completed/failed`. Done.
6. Record structured logs for phases: validate, fetch metadata, dedupe, download, write file, index image. Done.

Acceptance:

- Task creation persists `pending`. Done: `req_task_002_repository_persists_task_items_and_logs`.
- Worker wrapper moves task to `running`. Done: `req_task_001_single_download_task_completes_and_links_image`.
- Success moves task to `completed`. Done.
- Failure moves task to `failed` with error code/message. Done: `req_task_004_single_download_task_records_failure_diagnostics`.
- Progress is monotonic. Done: `req_task_003_req_task_005_enforces_explicit_and_monotonic_task_transitions`.
- Logs are queryable by task ID. Done.

### Phase 2D: Test and Script Expansion

Requirements: testing strategy and traceability rules.

Status: complete.

Tasks:

1. Add `tests/integration/backend_sqlite.sh`. Done.
2. Add repository integration tests. Done through deterministic backend SQLite integration script.
3. Extend `tests/run_local.sh` to run unit + integration tests. Done.
4. Extend live E2E to verify DB indexing after download. Done early for single-work E2E.

Acceptance:

- `./tests/run_local.sh` remains the default local quality gate. Done.
- Live E2E remains opt-in and credential-safe. Done.
- Every new test references requirement IDs where practical. Done for Rust tests; scripts group those tests by phase/layer.

### Phase 3A: Axum API Wrapper

Requirements: `REQ-DL-001`, `REQ-TASK-001` through `REQ-TASK-005`, `REQ-UI-002`, `REQ-UI-003`

Goal: expose the existing downloader/task core through thin HTTP endpoints without duplicating business logic.

Status: complete.

Tasks:

1. Add Axum dependency and backend server entrypoint. Done.
2. Add app state containing SQLite connection path / storage root / Pixiv client factory. Done.
3. Implement `POST /api/download/single`. Done.
4. Implement `GET /api/tasks/{task_id}`. Done.
5. Add API smoke/integration tests for success, validation failure, and task polling. Done.
6. Keep live Pixiv API tests opt-in and credential-safe. Done.

Acceptance:

- Single download API creates/enqueues a task and returns a `task_id`. Done: `req_dl_001_req_task_001_post_single_download_enqueues_and_returns_task_id`.
- Task query returns status, progress, item, log, and failure fields. Done: `req_task_002_req_task_004_get_task_returns_items_and_logs`.
- API validation rejects malformed single-download input. Done: `req_ui_002_post_single_download_rejects_invalid_pixiv_id`.
- API layer stays thin and delegates to `tasks` / `downloads`. Done: `post_download_single` calls `run_single_download_task`; `get_task` calls `TaskRepository`.
- `./tests/run_local.sh` remains green. Done.

### Phase 3B: Background Task Queue

Requirements: `REQ-TASK-001` through `REQ-TASK-005`, `REQ-DL-001`, `REQ-UI-003`

Goal: make the API's asynchronous task contract real. `POST /api/download/single` should create/enqueue a task and return quickly with `202 Accepted`, while a background worker runs the existing task/download core and `GET /api/tasks/{task_id}` exposes durable progress.

Status: complete.

Tasks:

1. Design the queue/worker boundary and keep the API layer thin. Done.
2. Add a task enqueue path that persists `pending` before worker execution. Done.
3. Add an in-process Tokio worker for single downloads. Done.
4. Ensure worker code delegates to existing `tasks` / `downloads` modules. Done.
5. Preserve current API DTOs where possible. Done; `image_id` is null and `download_status` is `pending` at enqueue time.
6. Add deterministic API/queue tests for enqueue, polling, completion, and failure diagnostics. Done.
7. Keep live Pixiv tests opt-in and credential-safe. Done.
8. Run `./tests/run_local.sh`. Done.
9. Cleanup check: generate a cleanup candidate list and wait for confirmation before any deletion. Done; no deletion performed.

Acceptance:

- `POST /api/download/single` returns `202` with `task_id` before the download has to finish. Done.
- Worker transitions task state through durable `pending -> running -> completed/failed`. Done.
- `GET /api/tasks/{task_id}` can observe task progress and terminal diagnostics. Done.
- Existing DB-aware downloader and task persistence remain the single source of business logic. Done.
- No Pixiv cookie or secret is written to code, docs, fixtures, or tests. Done.
- `./tests/run_local.sh` remains green. Done.

### Phase 4A: Frontend Scaffold

Requirements: `REQ-UI-001` through `REQ-UI-006`, `REQ-THEME-001` through `REQ-THEME-005`

Goal: create a usable Next.js shell that can receive real downloader/task APIs incrementally.

Status: complete.

Acceptance:

- Next.js app shell and routes exist for Home / Download / Tasks / Gallery / Settings. Done.
- Cyan Studio visual direction is implemented in global styles. Done.
- Download page can enqueue real single-download API tasks. Done.
- Tasks page can poll task-by-id. Done.
- `tests/stage/frontend_scaffold.sh` runs typecheck and production build. Done.

### Phase 4B: Data API Wiring

Requirements: `REQ-IMG-002`, `REQ-IMG-003`, `REQ-CFG-001`, `REQ-SEC-001`, `REQ-TASK-002`, `REQ-UI-003`, `REQ-UI-004`, `REQ-UI-005`

Goal: add minimum viable gallery/settings/task-list APIs and replace the largest frontend placeholders with real data.

Status: complete.

Tasks:

1. Add gallery metadata repository query with tag/category/source/R18 filters and cursor pagination. Done.
2. Add settings repository allowlist save with secret masking and masked-secret retention. Done.
3. Add task list repository query with status/type filters and cursor pagination. Done.
4. Add thin Axum endpoints for images, settings, and task list. Done.
5. Wire Gallery, Settings, and Tasks frontend pages to the new APIs. Done.
6. Add deterministic Phase 4B script. Done: `tests/stage/phase4b_data_api.sh`.
7. Run `./tests/run_local.sh`. Done.

Acceptance:

- `GET /api/images` returns indexed image metadata and tags without exposing local filesystem paths. Done.
- `GET /api/images/{image_id}` returns full metadata, tags, and source history. Done.
- `GET /api/settings` returns public settings with secrets masked. Done.
- `PUT /api/settings/{key}` saves known settings through repository validation. Done.
- `GET /api/tasks` returns recent task summaries for the Tasks page. Done.
- Frontend Gallery / Settings / Tasks now use real API data. Done.
- Live Pixiv tests remain opt-in and credential-safe. Done.

### Phase 4C: Frontend-Configured Single Download

Requirements: `REQ-DL-001`, `REQ-DL-007`, `REQ-CFG-001`, `REQ-CFG-002`, `REQ-SEC-001`, `REQ-UI-002`, `REQ-UI-004`

Goal: make the first real manual browser test possible: configure Pixiv credential and download directory in Settings, submit a single-work download from Download, inspect task progress in Tasks, then see indexed metadata in Gallery.

Status: complete.

Tasks:

1. Make the downloader runtime read `pixiv_cookie` from settings, with `PIXIV_PHPSESSID` as a runtime fallback. Done.
2. Make the worker resolve `download_base_path` from settings instead of only using startup `PIXIV_DOWNLOAD_ROOT`. Done.
3. Add path validation/normalization for settings-backed download roots, including `~` expansion and directory creation. Done.
4. Keep API responses from exposing local filesystem paths. Done.
5. Add `POST /api/settings/test/pixiv` for opt-in credential validation without returning the cookie. Done.
6. Update Settings frontend controls with Pixiv connection test. Done.
7. Add deterministic tests for settings-backed cookie/root resolution using mock Pixiv data. Done.
8. Run `./tests/run_local.sh` and update docs. Done.

Acceptance:

- Settings-saved `pixiv_cookie` is used by the single-download enqueue/worker path. Done.
- Settings-saved `download_base_path` is used as the storage root for subsequent single downloads. Done.
- Missing Pixiv credential fails before enqueue with `MISSING_PIXIV_COOKIE`. Done.
- Pixiv connection test can validate configured credentials without returning secret values. Done.
- No API response exposes the local filesystem path. Done.

### Phase 4D: Gallery File Preview Slice

Requirements: `REQ-IMG-004`, `REQ-SEC-002`, `REQ-UI-005`

Goal: make downloaded files visible in Gallery without exposing local filesystem paths.

Status: complete.

Tasks:

1. Add image file lookup helper in the repository/query layer. Done.
2. Add secure `GET /api/images/{image_id}/file` byte endpoint. Done.
3. Add `preview_url` / `thumbnail_url` values to gallery DTOs. Done.
4. Wire Gallery cards and detail view to real image bytes. Done.
5. Add deterministic Phase 4D script. Done: `tests/stage/phase4d_gallery_file_api.sh`.

Acceptance:

- Existing local files return image bytes and content type. Done.
- Unknown or unsafe image paths return stable errors. Done.
- Metadata JSON does not expose local filesystem paths. Done.
- Gallery previews render through the secure file endpoint. Done.

### Phase 5A: Author Batch Download Slice

Requirements: `REQ-DL-003`, `REQ-DL-006`, `REQ-CFG-004`, `REQ-CFG-005`, `REQ-CFG-007`, `REQ-TASK-001` through `REQ-TASK-005`, `REQ-UI-002`, `REQ-UI-003`

Goal: prove the first traceable multi-item download source by author UID while reusing the existing DB-aware downloader.

Status: complete.

Tasks:

1. Add `default_batch_count` setting with default `20`. Done.
2. Add Pixiv author discovery method to the client trait and mock client. Done.
3. Implement `POST /api/downloads/author`. Done.
4. Create author tasks with one `task_items` row per discovered work. Done.
5. Extend worker dispatch to run author tasks sequentially. Done.
6. Reuse the DB-aware single-work downloader for each author item. Done.
7. Preserve partial failures as item diagnostics and terminal `completed_with_errors`. Done.
8. Wire Download -> Author form to the real API. Done.
9. Add deterministic Phase 5A script. Done: `tests/stage/phase5a_author_batch.sh`.

Acceptance:

- Omitted limit uses `default_batch_count`; explicit limit above `max_request_count` is rejected. Done.
- Missing Pixiv credential fails before enqueue. Done.
- Multi-item author task updates task items and progress. Done.
- Partial item failure does not erase successful items and ends as `completed_with_errors`. Done.
- Live author batch remains opt-in with tiny runtime limits and credentials only. Done.

### Phase 5B: Bookmarks Batch Download Slice

Requirements: `REQ-DL-002`, `REQ-DL-006`, `REQ-CFG-004`, `REQ-CFG-005`, `REQ-CFG-007`, `REQ-TASK-001` through `REQ-TASK-005`, `REQ-UI-002`, `REQ-UI-003`

Goal: prove current-user collection acquisition while reusing the Phase 5A batch task pipeline.

Status: complete.

Tasks:

1. Add Pixiv bookmark discovery method to the client trait and mock client. Done.
2. Implement `POST /api/downloads/bookmarks`. Done.
3. Create bookmark tasks with one `task_items` row per discovered work. Done.
4. Extend worker dispatch to run bookmark tasks sequentially. Done.
5. Reuse the DB-aware single-work downloader for each bookmarked item. Done.
6. Preserve partial failures as item diagnostics and terminal `completed_with_errors`. Done.
7. Wire Download -> Bookmarks form to the real API. Done.
8. Add deterministic Phase 5B script. Done: `tests/stage/phase5b_bookmark_batch.sh`.

Acceptance:

- Omitted limit uses `default_batch_count`; explicit limit above `max_request_count` is rejected. Done.
- Missing Pixiv credential fails before enqueue. Done.
- Multi-item bookmark task updates task items and progress. Done.
- Source history is recorded as `bookmark`. Done.
- Partial item failure does not erase successful items and ends as `completed_with_errors`. Done.
- Live bookmark batch remains opt-in with tiny runtime limits and credentials only. Done.

## Deferred Until After Phase 5B

| Area | Reason |
| --- | --- |
| Generated thumbnail cache | Preview now works through secure original-file serving; cached thumbnails can follow when performance requires it |
| Top10/random batch modes | Author/bookmark batches now prove the shared task-item path; remaining sources need source-specific discovery |
| DeepSeek smart retrieval | Needs stable download/task substrate first |
| DeepSeek connection tests | Need credential-safe live checks with opt-in runtime secrets |
| Map search and image editing | Needs richer gallery query/edit APIs first |

## Backend API Completion Snapshot

Rough status as of Phase 5B:

- Single-work downloader/task API: mature for current vertical slice.
- Author batch downloader/task API: minimum viable and tested with deterministic mocks.
- Bookmark batch downloader/task API: minimum viable and tested with deterministic mocks.
- Task query/list API: minimum viable and tested.
- Gallery metadata, secure file preview, and hard-delete API: minimum viable and tested.
- Settings public list/save API: minimum viable and tested, including secret masking and settings-backed single-download runtime.
- Pixiv connection test API: minimum viable and tested with mock Pixiv data.
- Full product API: partial. About 14 endpoint families are implemented out of the planned API surface, while top10/random batch modes, generated thumbnail caching, task cancellation, image editing, and map APIs remain pending.

Practical meaning:

- The backend is solid enough to keep wiring real frontend flows.
- The backend is not yet complete enough for a full product demo.
- The next API work should target the exact frontend flow being tested, not add broad unused endpoints.

## Immediate Next Implementation Step

Recommended next stage: **Phase 7B follow-up - Gallery Quality / Thumbnail Cache Slice**.

Reason:

- The manual single-download flow, Gallery preview loop, author batch flow, bookmark batch flow, and smart batch flow now work through the same downloader-first substrate.
- Batch and smart retrieval will quickly increase local image volume, so Gallery needs thumbnail caching and browsing ergonomics before adding more discovery sources.
- Gallery already has secure preview and hard-delete, making it the best next place to improve product feel without destabilizing downloader core.
- Top10 / Random remain useful, but they add more ingestion volume before the browsing surface is ready.

Gallery Delete completed:

1. Hard delete semantics selected for V1.
2. Repository/query logic resolves image paths safely, deletes files under allowed roots, and updates image/tag/source indexes consistently.
3. Thin API endpoints exist for single and batch delete.
4. Gallery selection controls and refresh behavior are wired to the real delete endpoint.
5. Deterministic tests cover existing file deletion, missing-file behavior, unsafe path rejection, DB index cleanup, and batch per-item results.

Phase 6A completed:

1. Added DeepSeek settings and connection test without storing secrets in repository files.
2. Added LLM parse endpoint that converts natural language into structured tags, negative tags, count, R18 policy, confidence, and model.
3. Added frontend preview step for the generated tag set.
4. Kept live Pixiv and live LLM checks opt-in; deterministic tests use mocks.

Phase 6B completed:

1. Add Pixiv tag search discovery through the Pixiv client abstraction.
2. Create `smart` batch task from the accepted tag plan.
3. Persist smart retrieval provenance in `smart_retrievals`.
4. Wire the Download Smart form from parse preview to enqueue.
5. Add deterministic tests for tag search, count caps, source history, provenance, and partial failures.

Phase 7A completed:

1. Replace the placeholder Home page with a real utility dashboard.
2. Reuse `GET /api/tasks`, `GET /api/images`, and `GET /api/settings`; no new backend API was needed.
3. Show recent task status, recent downloaded image previews, quick entries, download path/library state, and masked Pixiv/DeepSeek configuration status.
4. Add frontend scaffold checks that assert Home remains wired to the real APIs.
5. Verify the dashboard in the local browser against the live local backend/frontend.

Phase 7B UI polish completed:

1. Convert Download Center to a balanced Single / Author / Bookmarks / Smart tabbed workbench.
2. Move Gallery image detail from a bottom preview area to a right-side drawer with preview, metadata, tags, sources, and close control.
3. Move Tasks detail into a centered modal, keep Recent Tasks at 10 by default, and support expanding more.
4. Group Settings into categorized panels without exposing secret values.
5. Add a Home recent normal image banner while keeping the page a practical workbench.
6. Add deterministic frontend scaffold anchors for the polish pass; no new backend API was needed.

Recommended next planning options:

1. Phase 7B follow-up: Gallery thumbnail cache, richer filters, and browsing quality pass.
2. Phase 7C: Top10 / random discovery modes using the existing batch task template.
3. Phase 7D: Task operations: cancel/retry and clearer worker diagnostics.

Phase 7B follow-up proposed todo:

1. Backend thumbnail endpoint/cache: generate small local thumbnails under the download root or a cache directory without exposing raw paths.
2. Gallery API response: prefer `thumbnail_url` when available and keep `preview_url` for full image display.
3. Frontend Gallery: render thumbnails in the grid, keep full preview/detail on selection, improve empty/loading/error states.
4. Filters: tighten existing tag/category/source/R18 filters and add source chips for single/bookmark/author/smart.
5. Tests: deterministic thumbnail generation/cache tests, API file-safety tests, Gallery frontend typecheck/build.
6. Docs: update progress, API contract, traceability, and cleanup candidates after implementation.

Optional batch-source backlog:

1. Phase 5C: Top10 refresh/download.
2. Phase 5D: random surprise.
3. Later: generated thumbnails, richer gallery edit/map APIs.

# Traceability Matrix

This matrix keeps requirements traceable to specs, future implementation modules, and tests.

Implementation paths are proposed for the planned frontend/backend split:

- Backend root: `src/backend`
- Frontend root: `src/frontend`

## Requirement to Design Mapping

| Requirement | Design Docs | Proposed Backend Modules | Proposed Frontend Areas | Proposed Tests |
| --- | --- | --- | --- | --- |
| `REQ-DL-001` | `api-contract.md`, `task-flow.md`, `architecture.md`, `pixiv-client.md` | `src/backend/src/downloads/mod.rs`, `src/backend/src/pixiv/mod.rs`, `src/backend/src/api.rs` | `/download` Single Work tab | `req_dl_001_downloads_single_work_with_mock_pixiv`, `req_dl_001_req_img_001_db_aware_first_download_indexes_file_tags_and_source`, `req_dl_001_req_task_001_post_single_download_enqueues_and_returns_task_id`, `req_dl_001_req_cfg_002_single_download_uses_settings_cookie_and_download_root` |
| `REQ-DL-002` | `api-contract.md`, `task-flow.md`, `phase5-batch-download-plan.md` | `src/backend/src/tasks/mod.rs`, `src/backend/src/pixiv/mod.rs`, `src/backend/src/api.rs` | `/download` Bookmarks tab | `req_dl_002_bookmark_batch_task_completes_multiple_items`, `req_dl_002_post_bookmark_download_enqueues_and_uses_default_batch_count` |
| `REQ-DL-003` | `api-contract.md`, `database-schema.md`, `phase5-batch-download-plan.md` | `src/backend/src/tasks/mod.rs`, `src/backend/src/pixiv/mod.rs`, `src/backend/src/api.rs` | `/download` Author tab | `req_dl_003_author_batch_task_completes_multiple_items`, `req_dl_003_post_author_download_enqueues_and_uses_default_batch_count` |
| `REQ-DL-004` | `api-contract.md`, `frontend-spec.md` | `downloads/top10.rs`, `ranking/cache.rs` | `/` Top10 carousel, `/download` Top10 tab | top10 refresh/cache tests |
| `REQ-DL-005` | `api-contract.md`, `frontend-spec.md` | `downloads/random.rs` | `/` Random Surprise, `/download` Random tab | random task creation tests |
| `REQ-DL-006` | `domain-model.md`, `database-schema.md`, `file-storage.md` | `src/backend/src/downloads/mod.rs`, `src/backend/src/storage/mod.rs`, `src/backend/migrations/0001_init.sql`, `src/backend/src/images/mod.rs` | Gallery duplicate-safe display | `req_dl_006_skips_existing_file_as_duplicate`, `req_dl_006_migration_enforces_unique_pixiv_page_identity`, `req_dl_006_db_duplicate_skip_avoids_image_download_and_records_source`, `req_dl_006_missing_file_repair_redownloads_and_refreshes_index`, `req_dl_006_existing_file_indexing_inserts_db_without_downloading_bytes` |
| `REQ-DL-007` | `requirements.md`, `frontend-spec.md` | `src/backend/src/api.rs`, `src/backend/src/settings/mod.rs`, `pixiv/auth.rs` | `/settings` Pixiv connection | `req_dl_007_settings_backed_pixiv_cookie_is_required_before_enqueue`, `req_dl_007_settings_pixiv_test_uses_masked_cookie_without_download`, masked settings tests |
| `REQ-AI-001` | `api-contract.md`, `domain-model.md` | `src/backend/src/ai.rs`, `src/backend/src/api.rs` | `/download` Smart parse panel | `req_ai_001_parses_and_normalizes_deepseek_json_plan`, `req_ai_001_post_smart_parse_returns_structured_tag_plan` |
| `REQ-AI-002` | `api-contract.md`, `task-flow.md` | `src/backend/src/tasks/mod.rs`, `src/backend/src/pixiv/mod.rs`, `src/backend/src/api.rs` | `/download` Smart submit flow | `req_ai_002_parse_search_results_ignores_user_and_bookmark_ids`, `req_ai_002_smart_batch_task_completes_multiple_items_and_provenance`, `req_ai_002_post_smart_download_enqueues_tag_search_task` |
| `REQ-AI-003` | `database-schema.md`, `domain-model.md` | `src/backend/src/tasks/mod.rs`, `src/backend/migrations/0001_init.sql` | Smart result review | `req_ai_002_smart_batch_task_completes_multiple_items_and_provenance` |
| `REQ-AI-004` | `frontend-spec.md`, `api-contract.md` | `src/backend/src/api.rs` | Count override control | `req_cfg_004_post_smart_download_rejects_count_above_max_request_count` |
| `REQ-AI-005` | `task-flow.md`, `api-contract.md` | `src/backend/src/ai.rs`, `src/backend/src/api.rs`, `src/backend/src/errors.rs` | Smart parse error state | `req_ai_005_rejects_empty_deepseek_tag_plan`, `req_ai_005_post_smart_parse_requires_deepseek_key_before_parse` |
| `REQ-IMG-001` | `database-schema.md`, `domain-model.md` | `src/backend/migrations/0001_init.sql`, `src/backend/src/db/mod.rs`, `src/backend/src/images/mod.rs`, `src/backend/src/downloads/mod.rs` | Gallery metadata display | `req_img_001_migration_creates_core_image_tables`, `req_img_001_repository_inserts_and_queries_image_metadata`, `req_dl_001_req_img_001_db_aware_first_download_indexes_file_tags_and_source` |
| `REQ-IMG-002` | `api-contract.md`, `frontend-spec.md` | `src/backend/src/images/mod.rs`, `src/backend/src/api.rs` | `/gallery` filters | `req_img_002_req_img_003_repository_lists_images_with_filters_and_cursor`, `req_img_002_req_ui_005_get_images_returns_gallery_metadata` |
| `REQ-IMG-003` | `frontend-spec.md`, `api-contract.md` | `src/backend/src/images/mod.rs` cursor pagination, `src/backend/src/api.rs` | `/gallery` lazy grid | `req_img_002_req_img_003_repository_lists_images_with_filters_and_cursor` |
| `REQ-IMG-004` | `api-contract.md`, `frontend-spec.md` | `src/backend/src/images/mod.rs`, `src/backend/src/api.rs` | `/gallery` right-side preview drawer | `req_img_002_req_ui_005_get_images_returns_gallery_metadata`, `req_img_004_req_sec_002_get_image_file_serves_bytes_without_path_leak`, `tests/stage/frontend_scaffold.sh` Phase 7B drawer anchor |
| `REQ-IMG-005` | `api-contract.md`, `domain-model.md` | `map/points.rs`, `images/repository.rs` | `/gallery` map panel | map point query tests |
| `REQ-IMG-006` | `api-contract.md`, `database-schema.md` | `images/update.rs`, `tags/repository.rs` | Tag/category edit controls | image patch tests |
| `REQ-IMG-007` | `api-contract.md`, `frontend-spec.md`, `file-storage.md` | `src/backend/src/images/mod.rs`, `src/backend/src/api.rs` | `/gallery` selection delete controls | `req_img_007_req_sec_002_delete_image_removes_file_and_index_rows`, `req_img_007_delete_batch_returns_per_item_results` |
| `REQ-TASK-001` | `task-flow.md`, `domain-model.md` | `src/backend/src/tasks/mod.rs`, `src/backend/src/api.rs`, future `tasks/queue.rs`, `tasks/worker.rs` | Task links from all submit flows | `req_task_001_single_download_task_completes_and_links_image`, `req_dl_001_req_task_001_post_single_download_enqueues_and_returns_task_id` |
| `REQ-TASK-002` | `api-contract.md`, `task-flow.md` | `src/backend/migrations/0001_init.sql`, `src/backend/src/tasks/mod.rs`, `src/backend/src/api.rs`, future `tasks/handlers.rs` | `/tasks`, task indicator, recent list, task detail modal | `req_task_002_migration_creates_task_traceability_tables`, `req_task_002_repository_persists_task_items_and_logs`, `req_task_002_repository_lists_tasks_with_filters_and_cursor`, `req_task_002_req_task_004_get_task_returns_items_and_logs`, `req_task_002_req_ui_003_get_tasks_returns_task_list`, `tests/stage/frontend_scaffold.sh` Phase 7B modal/recent-limit anchors |
| `REQ-TASK-003` | `domain-model.md`, `task-flow.md` | `src/backend/src/tasks/mod.rs`, future `tasks/state_machine.rs` | `/tasks` status labels | `req_task_003_req_task_005_enforces_explicit_and_monotonic_task_transitions` |
| `REQ-TASK-004` | `task-flow.md`, `database-schema.md` | `src/backend/src/tasks/mod.rs`, `src/backend/src/errors.rs`, `src/backend/src/api.rs` | Task detail logs | `req_task_004_single_download_task_records_failure_diagnostics`, `req_task_002_req_task_004_get_task_returns_items_and_logs`, `req_task_004_queued_single_download_preserves_failure_diagnostics` |
| `REQ-TASK-005` | `domain-model.md`, `task-flow.md` | `src/backend/src/tasks/mod.rs` | Progress bars | `req_task_003_req_task_005_enforces_explicit_and_monotonic_task_transitions` |
| `REQ-CFG-001` | `api-contract.md`, `frontend-spec.md` | `src/backend/src/settings/mod.rs`, `src/backend/src/api.rs`, future `settings/handlers.rs`, `settings/secrets.rs` | `/settings` Pixiv section | `req_cfg_001_settings_repository_upserts_existing_value`, `req_cfg_001_req_sec_001_settings_repository_saves_known_values_and_masks_secret`, `req_cfg_001_settings_repository_rejects_unknown_or_invalid_values`, `req_cfg_001_req_sec_001_settings_api_lists_and_saves_masked_values` |
| `REQ-CFG-002` | `database-schema.md`, `frontend-spec.md` | `src/backend/src/api.rs`, `src/backend/src/settings/mod.rs`, `files/path.rs` | `/settings` Storage section | `req_dl_001_req_cfg_002_single_download_uses_settings_cookie_and_download_root`, settings path validation tests |
| `REQ-CFG-003` | `api-contract.md`, `frontend-spec.md` | `src/backend/src/settings/mod.rs`, `src/backend/src/ai.rs`, `src/backend/src/api.rs` | `/settings` DeepSeek section | `req_cfg_003_settings_repository_reads_default_deepseek_model_publicly`, `req_cfg_003_settings_deepseek_test_uses_masked_key_without_exposing_secret` |
| `REQ-CFG-004` | `requirements.md`, `api-contract.md` | `src/backend/src/api.rs` | Quantity controls | `req_cfg_004_post_author_download_rejects_limit_above_max_request_count`, `req_cfg_004_post_bookmark_download_rejects_limit_above_max_request_count`, `req_cfg_004_post_smart_parse_rejects_count_above_max_request_count`, `req_cfg_004_post_smart_download_rejects_count_above_max_request_count` |
| `REQ-CFG-005` | `domain-model.md`, `frontend-spec.md`, `visual-theme.md` | `src/backend/src/domain.rs`, future `settings/r18.rs`, `images/query.rs` | R18 controls across pages | `req_cfg_005_skips_r18_when_policy_excludes_it`, R18 filtering tests |
| `REQ-CFG-006` | `frontend-spec.md`, `visual-theme.md` | `settings/theme.rs` | Theme selector | theme persistence tests |
| `REQ-CFG-007` | `requirements.md`, `api-contract.md`, `phase5-batch-download-plan.md` | `src/backend/src/settings/mod.rs`, `src/backend/src/api.rs` | Batch count defaults | `req_dl_003_post_author_download_enqueues_and_uses_default_batch_count`, `req_dl_002_post_bookmark_download_enqueues_and_uses_default_batch_count` |
| `REQ-UI-001` | `frontend-spec.md`, `visual-theme.md` | Existing `GET /api/tasks`, `GET /api/images`, `GET /api/settings`; future Top10 endpoints | `/` Home dashboard, normal/wide image banner, Rust/performance panels | `tests/stage/frontend_scaffold.sh` Home dashboard/banner/performance wiring check |
| `REQ-UI-002` | `frontend-spec.md`, `api-contract.md` | `src/backend/src/api.rs` download endpoints | `/download` tabbed workbench and Smart tag chips | `req_ui_002_post_single_download_rejects_invalid_pixiv_id`, `tests/stage/frontend_scaffold.sh` Phase 7B download tabs/tag-chip/API unwrap anchors |
| `REQ-UI-003` | `frontend-spec.md`, `api-contract.md` | `src/backend/src/api.rs` task endpoints | `/tasks` recent list and closable task detail modal | `req_task_002_req_task_004_get_task_returns_items_and_logs`, `req_task_002_req_ui_003_get_tasks_returns_task_list`, `tests/stage/frontend_scaffold.sh` Phase 7B task modal anchors |
| `REQ-UI-004` | `frontend-spec.md`, `api-contract.md` | `src/backend/src/api.rs`, `src/backend/src/settings/mod.rs` | `/settings` categorized panels | `req_cfg_001_req_sec_001_settings_api_lists_and_saves_masked_values`, `tests/stage/frontend_scaffold.sh` Phase 7B settings category anchor |
| `REQ-UI-005` | `frontend-spec.md`, `api-contract.md` | `src/backend/src/api.rs`, `src/backend/src/images/mod.rs`, future map endpoints | `/gallery` grid and closable right-side detail drawer | `req_img_002_req_ui_005_get_images_returns_gallery_metadata`, `tests/stage/frontend_scaffold.sh` Phase 7B gallery drawer anchor |
| `REQ-UI-006` | `frontend-spec.md` | Settings/filter endpoints | App shell, URL state | navigation state tests |
| `REQ-THEME-001` | `visual-theme.md`, `frontend-spec.md` | Theme settings endpoint | Global CSS variables | visual regression checks |
| `REQ-THEME-002` | `visual-theme.md` | Theme settings endpoint | Theme preview component | theme option tests |
| `REQ-THEME-003` | `visual-theme.md` | N/A | CSS theme tokens | CSS token review |
| `REQ-THEME-004` | `visual-theme.md`, `frontend-spec.md` | N/A | Download/tasks/settings layouts | responsive UI tests |
| `REQ-THEME-005` | `visual-theme.md`, `domain-model.md` | `settings/r18.rs`, `images/query.rs` | R18 blur/visibility UI | R18 visual state tests |
| `REQ-SEC-001` | `requirements.md`, `api-contract.md`, `database-schema.md` | `src/backend/src/settings/mod.rs`, `src/backend/src/api.rs`, future `settings/secrets.rs` | Settings masked states | `req_sec_001_settings_repository_masks_secret_values`, `req_sec_001_settings_repository_masks_secrets_in_public_list`, `req_cfg_001_req_sec_001_settings_repository_saves_known_values_and_masks_secret`, `req_cfg_001_req_sec_001_settings_api_lists_and_saves_masked_values` |
| `REQ-SEC-002` | `api-contract.md`, `file-storage.md` | `src/backend/src/storage/mod.rs`, future `images/files.rs`, DTO mapping | Gallery/file preview | `req_sec_002_rejects_unsafe_pixiv_id_path_segments`, path exposure tests |
| `REQ-SEC-003` | `domain-model.md`, `database-schema.md` | `images/repository.rs`, `settings/r18.rs` | Gallery/home filtering | category persistence tests |

## Open Decisions

| Decision | Current Direction | Reason |
| --- | --- | --- |
| `doc` vs `docs` | Use existing `docs` | Repository already contains `docs/product_requirements.md` |
| `frontend` spelling | Use `src/frontend` | Conventional name; avoids future confusion |
| Multi-page Pixiv work identity | `pixiv_id + page_index` | Pixiv works may contain multiple images |
| Tags storage | Normalized `image_tags` plus array DTO | Query performance and ergonomic API |
| Partial batch failure status | Add `completed_with_errors` | More traceable than only `completed` or `failed` |
| Cancellation | Reserve in state machine, optional V1 implementation | Useful state, but not mandatory first milestone |
| Default visual theme | `cyan-studio` | User selected Demo B |
| Implementation priority | Downloader-first | User emphasized core download script as most important |
| V1 file layout | `{download_root}/originals/{pixiv_id}/{pixiv_id}_p{page}.{ext}` | Keeps source/category mutable in SQLite |
| Test script baseline | `tests/run_local.sh`, `tests/unit/backend_unit.sh`, `tests/e2e/live_single_download.sh` | Keeps unit and e2e flows repeatable |
| Current phase | Phase 7B UI Layout / Interaction Polish first pass complete; Gallery thumbnail cache remains recommended next | See `docs/progress.md` |
| Manual frontend anchor | Single Pixiv ID, Author Batch, and Bookmarks Batch downloads succeed from browser as of 2026-05-22; Smart Retrieval parse/edit/enqueue is manually validated as of 2026-05-23; Home Dashboard loaded real local task/image/settings data on 2026-05-23 | Confirms Settings -> Download -> Tasks -> Gallery -> Home workbench path |

## Current Milestone Order

1. Completed: specs and downloader-first architecture skeleton.
2. Completed: backend core scaffold and single-work downloader.
3. Completed: SQLite migrations, image repositories, settings repository.
4. Completed: DB-aware downloader and DB dedupe.
5. Completed: task state persistence and logs.
6. Completed: test and script expansion.
7. Completed: Axum API wrapper for single download and task polling.
8. Completed: background queue hardening.
9. Completed: frontend scaffold.
10. Completed: Phase 4B gallery/settings/task-list data API wiring.
11. Completed: Phase 4C frontend-configured single download.
12. Completed: Phase 4D secure gallery file serving plus frontend previews.
13. Completed: Phase 5A author batch download task.
14. Completed: Phase 5B bookmarks batch download task.
15. Completed: Phase 4E Gallery hard-delete file/index cleanup.
16. Completed: Phase 6A DeepSeek smart retrieval parse slice.
17. Completed: Phase 6B smart tag search and batch download.
18. Completed: Phase 7A Home Dashboard.
19. Completed: Phase 7B UI Layout / Interaction Polish first pass.
20. Current next choice: Gallery thumbnail cache and browsing quality pass.

## Traceability Rule for Future Code

When implementing a feature, add at least one of:

- Test name includes the requirement ID.
- Module-level comment references the requirement ID.
- Migration comment references the requirement ID.
- Route handler documentation references the requirement ID.

This keeps the project easy to audit without over-commenting every line.

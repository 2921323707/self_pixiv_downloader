# Tests

This folder contains cross-project test entrypoints, fixtures, smoke scripts, e2e scripts, and live-check notes.

Current state:

- Backend core scaffold exists at `src/backend`.
- Frontend scaffold exists and Home / Download / Gallery / Settings / Tasks now use real data APIs plus Phase 7B UI polish/follow-up anchors; this forms the v1.0.0 downloader-first final test surface.
- Runnable backend unit, stage, deterministic integration, API smoke, Phase 3B queue, Phase 4B data API, Phase 4C configured download, Phase 4D gallery file API, Phase 4E gallery delete, Phase 5A author batch, Phase 5B bookmark batch, Phase 6A smart parse, Phase 6B smart download, and frontend scaffold checks are available. The frontend scaffold check also asserts Phase 7B Home banner selection, Home Rust/performance panels, Download tabs/tag chips/API unwrap guard, Gallery drawer close hooks, Tasks modal/recent-list close hooks, and Settings category anchors.
- Windows Web and Tauri App were manually validated on 2026-05-27. Windows local commands use `npm.cmd` and the helper scripts under `tools/`.

Run deterministic local tests:

```text
./tests/run_local.sh
```

Run backend unit tests directly:

```text
./tests/unit/backend_unit.sh
```

Run milestone-specific stage checks directly:

```text
./tests/stage/phase2a_repository.sh
./tests/stage/phase2c_tasks.sh
./tests/stage/phase3b_queue.sh
./tests/stage/phase4b_data_api.sh
./tests/stage/phase4c_configured_download.sh
./tests/stage/phase4d_gallery_file_api.sh
./tests/stage/phase5a_author_batch.sh
./tests/stage/phase5b_bookmark_batch.sh
./tests/stage/phase6a_smart_parse.sh
./tests/stage/phase6b_smart_download.sh
./tests/stage/frontend_scaffold.sh
```

Run deterministic backend integration checks:

```text
./tests/integration/backend_sqlite.sh
```

Run Axum API smoke checks:

```text
./tests/smoke/backend_api.sh
```

Run opt-in live Pixiv e2e:

```text
PIXIV_PHPSESSID=... ./tests/e2e/live_single_download.sh
```

Optional live e2e overrides:

```text
PIXIV_TEST_WORK_ID=144920810
PIXIV_TEST_DOWNLOAD_DIR=/private/tmp/pixiv_platform_e2e_144920810_...
PIXIV_TEST_DB_PATH=/private/tmp/pixiv_platform_e2e_144920810_.../pixiv_platform.sqlite3
```

The live E2E path is DB-aware:

1. It runs the real Pixiv single-work download through `run_single_download_task`.
2. It verifies the downloaded file exists and is non-empty.
3. It verifies SQLite contains the completed task, task item, task logs, image row, and source history.
4. It runs the same request a second time and expects `SkippedDuplicate` from DB/file state.

Testing rules:

- Do not commit Pixiv cookies or DeepSeek keys.
- Live Pixiv tests must be opt-in and use tiny limits.
- Mock Pixiv tests should be implemented before live download tests.
- Test names should reference requirement IDs when practical.

Expected future structure:

```text
tests/
  fixtures/
  integration/
  unit/
  stage/
  smoke/
  e2e/
  live/
```

Current structure:

| Path | Purpose |
| --- | --- |
| `tests/unit/backend_unit.sh` | Runs the full backend Rust test suite. |
| `tests/stage/phase2a_repository.sh` | Focused SQLite migration, image repository, and settings repository checks. |
| `tests/stage/phase2c_tasks.sh` | Focused task repository and task lifecycle checks. |
| `tests/stage/phase3b_queue.sh` | Focused enqueue-first API and background queue checks. |
| `tests/stage/phase4b_data_api.sh` | Focused gallery/settings/task-list repository and API checks. |
| `tests/stage/phase4c_configured_download.sh` | Focused settings-backed Pixiv cookie and download-root checks. |
| `tests/stage/phase4d_gallery_file_api.sh` | Focused secure Gallery file serving and preview URL checks. |
| `tests/stage/phase4e_gallery_delete.sh` | Focused Gallery hard-delete file and index cleanup checks. |
| `tests/stage/phase5a_author_batch.sh` | Focused author batch API, default count, limit, worker, and partial-failure checks. |
| `tests/stage/phase5b_bookmark_batch.sh` | Focused bookmark batch API, default count, limit, worker, and partial-failure checks. |
| `tests/stage/phase6a_smart_parse.sh` | Focused DeepSeek settings, connection test, and smart parse checks with deterministic mocks. |
| `tests/stage/phase6b_smart_download.sh` | Focused Pixiv smart tag search parsing, smart task worker, provenance, API enqueue, missing-cookie, and count limit checks. |
| `tests/stage/frontend_scaffold.sh` | Frontend Home dashboard wiring check, Phase 7B UI polish/follow-up anchors, TypeScript check, and production build. |
| `tests/integration/backend_sqlite.sh` | Deterministic DB-aware downloader + repository + task integration gate. |
| `tests/smoke/backend_api.sh` | Deterministic Axum API wrapper smoke checks with mock Pixiv data. |
| `tests/e2e/live_single_download.sh` | Opt-in live Pixiv E2E; requires runtime cookie. |
| `tests/live/README.md` | Human-readable live test notes. |

See `docs/specs/testing-strategy.md` for the full testing plan.

Current local baseline:

```text
./tests/run_local.sh
86 backend unit tests passed; Phase 2A checks passed; Phase 2C checks passed; backend SQLite integration checks passed; backend API smoke checks passed; Phase 3B queue checks passed; Phase 4B data API checks passed; Phase 4C configured download checks passed; Phase 4D gallery file API checks passed; Phase 4E gallery delete checks passed; Phase 5A author batch checks passed; Phase 5B bookmark batch checks passed; Phase 6A smart parse checks passed; Phase 6B smart download checks passed; frontend scaffold checks passed; 0 failed
```

Latest Windows focused baseline:

```text
cd src/backend && cargo test
86 backend tests passed

cd src/frontend && npm.cmd run lint
TypeScript check passed

cd tauri-app && npm.cmd run build
NSIS installer built at tauri-app/src-tauri/target/release/bundle/nsis/Pixiv Platform_1.2.0_x64-setup.exe
```

Latest focused frontend check:

```text
./tests/stage/frontend_scaffold.sh
frontend scaffold checks passed, including Phase 7B UI polish/follow-up anchors, TypeScript check, and production build
```

Current live smoke notes are in `tests/live/README.md`.

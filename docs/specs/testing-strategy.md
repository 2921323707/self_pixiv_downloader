# Testing Strategy Specification

The test strategy follows the spec-coding workflow: tests should reference requirement IDs where practical and should separate deterministic local tests from live Pixiv/DeepSeek checks.

## Test Roots

| Location | Purpose |
| --- | --- |
| `tests/` | Cross-project smoke tests, fixtures, and live-check scripts |
| `tests/unit` | Script entrypoints for deterministic unit checks |
| `tests/stage` | Current milestone-specific quality gates |
| `tests/integration` | Deterministic cross-module integration gates |
| `tests/e2e` | Opt-in end-to-end tests, including live Pixiv checks |
| `tests/smoke` | Fast cross-module smoke checks after API/frontend exist |
| `tests/fixtures` | Static mock data and sample responses |
| `src/backend` | Rust unit and integration tests after backend scaffold exists |
| `src/frontend` | Next.js component, route, and visual tests after frontend scaffold exists |

## Test Layers

### 1. Backend Unit Tests

Run with Rust tooling once backend exists.

Script:

```text
./tests/unit/backend_unit.sh
```

Current milestone script:

```text
./tests/stage/phase2a_repository.sh
./tests/stage/phase2c_tasks.sh
```

Current deterministic integration script:

```text
./tests/integration/backend_sqlite.sh
```

Current API smoke script:

```text
./tests/smoke/backend_api.sh
```

Current Phase 3B queue script:

```text
./tests/stage/phase3b_queue.sh
```

Current Phase 5A author batch script:

```text
./tests/stage/phase5a_author_batch.sh
```

Current Phase 5B bookmark batch script:

```text
./tests/stage/phase5b_bookmark_batch.sh
```

Targets:

- Task state transitions: `REQ-TASK-003`, `REQ-TASK-005`
- Quantity and request validation: `REQ-CFG-004`
- R18 policy decisions: `REQ-CFG-005`, `REQ-SEC-003`
- Dedup key generation: `REQ-DL-006`
- AI structured response parsing: `REQ-AI-001`, `REQ-AI-005`

### 2. Backend Integration Tests

Targets:

- SQLite migrations create expected tables/indexes.
- Task creation writes `pending` rows.
- Worker state changes are persisted.
- Image metadata and tags can be inserted/query-filtered.
- Secret settings are masked in API DTOs.
- DB-aware downloader writes image metadata/source history and handles duplicate/repair paths.
- Axum API wrapper delegates single download work to tasks/downloads and returns task polling snapshots.
- Background queue tests verify enqueue-first responses, task polling to completion, and failure diagnostics.

These should use a temporary SQLite database and temporary download directory.

Current script:

```text
./tests/integration/backend_sqlite.sh
```

Current API smoke:

```text
./tests/smoke/backend_api.sh
```

### 3. Mock Pixiv Tests

Targets:

- Single work download flow with mocked Pixiv metadata/file response.
- Author batch flow with mixed success and item failure. Done in Phase 5A.
- Bookmark batch flow with mixed success and item failure. Done in Phase 5B.
- Top10 batch flow with mixed success, duplicate, and item failure.
- Top10 refresh cache behavior.

Mock tests are required before live Pixiv tests so CI/local development can run without credentials.

### 4. Live Pixiv Smoke Tests

Live tests are manual or opt-in only.

Script:

```text
PIXIV_PHPSESSID=... ./tests/e2e/live_single_download.sh
```

Required user-provided inputs:

- Valid `PHPSESSID`
- One safe test Pixiv work ID
- Optional author UID
- Explicit R18 policy for the run

Rules:

- Never commit credentials.
- Never print full cookie values.
- Use a small limit, usually `1` to `3`.
- Write files only to a temporary test download directory unless user chooses otherwise.

Suggested environment variables:

```text
PIXIV_PHPSESSID=
PIXIV_TEST_WORK_ID=
PIXIV_TEST_AUTHOR_UID=
PIXIV_TEST_DOWNLOAD_DIR=
PIXIV_TEST_DB_PATH=
```

### 5. DeepSeek Smoke Tests

Live tests are opt-in only.

Required user-provided inputs:

- `DEEPSEEK_API_KEY`
- Optional model name if not using project default

Checks:

- Natural language prompt returns parseable JSON.
- Parsed output contains tags, count recommendation, and R18 policy.
- Failure states are readable when credentials are invalid.

### 6. Frontend Tests

Targets:

- Route rendering for `/`, `/download`, `/gallery`, `/tasks`, `/settings`.
- Smart retrieval parse/edit/submit state.
- Task status badge mapping.
- Gallery filter state in URL.
- R18 blurred/hidden/visible thumbnail behavior.
- Cyan Studio theme tokens apply correctly.

### 7. Visual Verification

For frontend implementation, use browser screenshots at:

- Desktop: 1440px width.
- Tablet: 900px width.
- Mobile: 390px width.

Minimum pages:

- Homepage
- Download Center
- Task Panel
- Gallery
- Settings

## Initial Test Milestone

Before attempting live download, implement and pass:

1. Settings validation and secret masking tests.
2. Task state machine tests.
3. SQLite migration tests.
4. Mock single-work download test.
5. Dedupe repository test.

Live Pixiv tests should come after these because credential/network failures otherwise hide local logic bugs.

## User Inputs Needed for Live Download

Minimum:

- `PIXIV_PHPSESSID`
- `PIXIV_TEST_WORK_ID`

Optional:

- `PIXIV_TEST_AUTHOR_UID`
- `PIXIV_TEST_DOWNLOAD_DIR`
- `DEEPSEEK_API_KEY`

Credentials should be supplied through environment variables or local settings at runtime, never committed to the repository.

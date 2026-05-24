# Phase 5 Batch Download Plan

Status: Phase 5A author batch and Phase 5B bookmarks batch complete and included in v1.0.0 Final Delivery. Phase 5C Top10 and Phase 5D Random are optional post-delivery evolution tracks.

Goal: extend the proven single-work pipeline into traceable multi-item acquisition tasks without weakening the downloader-first architecture.

In this project, "batch download" means any workflow that discovers multiple Pixiv works and feeds them into the same DB-aware download/task pipeline. The source can be explicit user input, authenticated Pixiv state, or AI-produced search parameters.

## Batch Source Taxonomy

| Source | User Input | Discovery Step | Download Step | Status |
| --- | --- | --- | --- | --- |
| Author works | Pixiv author UID | Fetch works by author | Existing DB-aware downloader | Phase 5A complete |
| Bookmarks | Limit, R18 policy/category | Fetch current user's bookmarks | Existing DB-aware downloader | Phase 5B complete |
| Daily Top10 | refresh/download mode | Fetch ranking list | Optional selected/top download | Optional post-delivery |
| Random surprise | R18 policy and optional count | Deterministic random/search strategy | Existing DB-aware downloader | Optional post-delivery |
| Natural language smart retrieval | Prompt, optional count override | DeepSeek parse -> tags/negative tags -> Pixiv search | Existing DB-aware downloader | Phase 6B complete |

Natural-language retrieval is grouped conceptually with batch acquisition, but implementation remains after the core batch task engine because it also depends on DeepSeek configuration, prompt parsing, and provenance persistence.

## Unified Quantity And Filter Policy

The batch endpoints should share the same quantity and safety rules.

Settings:

| Setting | Current/Planned | Default | Meaning |
| --- | --- | ---: | --- |
| `max_request_count` | Current | `100` | Hard per-request cap enforced by backend. Existing validation allows `1..=500`. |
| `default_batch_count` | Current | `20` | Used when a batch request omits `limit` or count. |
| `r18_policy` | Current | `exclude` | Default download/visibility policy when a request does not override it. |

Endpoint behavior:

- If a request omits `limit`, use `default_batch_count`.
- If `default_batch_count` is absent in older databases, fall back to `20`.
- If request `limit` exceeds `max_request_count`, reject with validation error rather than silently downloading more than configured.
- If request `limit` is below `1`, reject with validation error.
- `max_request_count` is a safety ceiling, not a recommendation.
- `r18_policy` can be overridden per request; if omitted, use the setting.
- Live Pixiv tests must use tiny explicit limits, usually `1` to `3`.

Initial image filtering:

- Phase 5A/5B/5C/5D use `r18_policy` as the first safety/type filter.
- Phase 6 smart retrieval adds tags, negative tags, and count override.
- Pixiv work-type filtering such as illustration/manga/ugoira is not required for the first batch slice unless the user explicitly prioritizes it.

## Phase 5 Strategy

Phase 5 should not implement every batch source at once. The first slice should prove the shared batch task engine, then add source-specific discovery modes.

Original recommended order:

1. **Phase 5A: Author batch vertical slice**
2. **Phase 5B: Bookmarks batch**
3. **Phase 5C: Daily Top10 refresh/download** optional after v1.0.0
4. **Phase 5D: Random surprise** optional after v1.0.0
5. **Phase 6: Smart retrieval from natural language** complete through Phase 6B

Author batch is the first target because it is explicit, easy to test from a user-provided `author_uid`, and exercises the important batch machinery without adding homepage ranking cache or current-user bookmark semantics.

## Phase 5A: Author Batch Vertical Slice

Status: complete.

Requirements: `REQ-DL-003`, `REQ-DL-006`, `REQ-TASK-001` through `REQ-TASK-005`, `REQ-UI-002`, `REQ-UI-003`.

User-facing behavior:

1. User opens Download -> Author.
2. User enters Pixiv author UID, optional limit, and optional R18 policy.
3. Backend validates Pixiv credentials, author UID, limit, and configured download root.
4. API creates an async task and returns `202 Accepted { task_id }`.
5. Worker discovers up to `limit` works for that author.
6. Worker downloads each work/page through the existing DB-aware downloader.
7. Tasks page shows progress and per-item failures.
8. Gallery shows successfully downloaded images with source `author` and real previews.

Backend scope:

- Add Pixiv client discovery method for author works, mocked first. Done.
- Add quantity resolution helper: request limit -> `default_batch_count` -> hard cap by `max_request_count`. Done.
- Add request policy resolution helper: request `r18_policy` -> settings `r18_policy`. Done.
- Add batch task request creation for `TaskType::Author`. Done.
- Store one `task_items` row per discovered Pixiv work/page candidate. Done.
- Process items sequentially in the existing background worker. Done.
- Reuse `download_single_with_db` or a small shared helper so dedupe, missing-file repair, and source history stay consistent. Done.
- Mark task `completed` when all items complete or are skipped safely. Done.
- Mark task `completed_with_errors` when at least one item fails but the task can continue. Done.
- Mark task `failed` only for task-level blockers such as invalid settings, auth failure before discovery, or systemic filesystem setup failure. Done.

Frontend scope:

- Make the Download -> Author tab submit to `POST /api/downloads/author`. Done.
- Reuse existing task polling and task detail UI. Done.
- Show the returned task in Tasks without adding a new dashboard abstraction. Done.
- Keep Top10 / Random tabs visibly planned until their backend slices land.

Test scope:

- Mock Pixiv author discovery returns a deterministic list of work refs. Done.
- Author batch creates task and task items. Done.
- Worker processes multiple items and updates monotonic progress. Done.
- Duplicate works are skipped/indexed through the existing DB-aware path.
- One item failure produces `completed_with_errors` and item-level diagnostics. Done.
- Missing Pixiv cookie fails before enqueue. Done.
- Limit is clamped/enforced by settings and request validation. Done.
- Omitted limit uses default batch count. Done.
- Request limit above `max_request_count` is rejected. Done.
- Omitted R18 policy uses settings default. Done.
- Live author batch remains opt-in with tiny limits and runtime credentials only. Done.

Out of scope for Phase 5A:

- Parallel downloads.
- Retry/backoff tuning.
- Task cancellation.
- Generated thumbnail cache.
- Smart retrieval.
- Full Top10 homepage carousel.

## Later Phase 5 Slices

### Phase 5B: Bookmarks

Status: complete.

Adds `POST /api/downloads/bookmarks`.

Key extra concerns:

- Current-user bookmark discovery depends on authenticated Pixiv state. Done.
- Uses the same `limit`, `default_batch_count`, `max_request_count`, and `r18_policy` rules as author batch. Done.
- Reuses the sequential task item worker and DB-aware downloader. Done.
- Records source history as `bookmark`. Done.
- Need category/R18 policy handling if Pixiv bookmark categories are exposed by the selected client strategy.
- Live tests remain opt-in with tiny limits. Done.

### Phase 5C: Daily Top10

Adds `POST /api/downloads/top10` and later homepage Top10 display.

Key extra concerns:

- Ranking refresh can be metadata-only before download-all.
- Cache/ranking table may be useful before a polished homepage carousel.
- Top10 defaults to ten ranking entries, but any download-all behavior still respects `max_request_count`.

### Phase 5D: Random Surprise

Adds `POST /api/downloads/random`.

Key extra concerns:

- Random strategy must be deterministic in tests.
- The source should still produce traceable task items and logs.
- Default random count should be `1`; optional multi-random can use the same batch count policy later.

## Phase 6: Smart Retrieval From Natural Language

Adds `POST /api/smart/parse` and `POST /api/smart/download`.

Status: implemented through Phase 6B. Smart Parse prefers Japanese Pixiv tags with English fallback, preserves the user-selected R18 policy, and Smart Download creates a `smart` task with `smart_retrievals` provenance.

User-facing behavior:

1. User enters natural language.
2. DeepSeek converts the prompt into tags, negative tags, recommended count, and R18 policy.
3. User can review/edit tags and override count.
4. Backend searches Pixiv by the final tags and creates a smart batch task.
5. Worker downloads discovered works through the same DB-aware batch pipeline.

Count behavior:

- LLM `count_recommend` is a recommendation only.
- User override wins when present.
- Final count is capped by `max_request_count`.
- If both LLM count and user override are absent, use `default_batch_count`.

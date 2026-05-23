# Spec Completeness Audit

Audit date: 2026-05-21

User-selected visual direction: Demo B / `cyan-studio`

## Current Coverage

| Area | Status | Notes |
| --- | --- | --- |
| Product requirements | Covered | PRD has been converted into stable `REQ-*` IDs. |
| Domain model | Mostly covered | Core entities, enums, task states, image intake states are defined. |
| Async task flow | Covered | Queue lifecycle, polling, logs, and failure classes are specified. |
| Database schema | Mostly covered | SQLite tables, indexes, settings, provenance, and dedupe are specified. |
| API contract | Mostly covered | Main backend/frontend contracts are defined. |
| Frontend pages | Covered | Home, Download, Tasks, Gallery, Settings are specified. |
| Visual theme | Covered | Demo B is now selected as default `cyan-studio`. |
| Traceability | Covered | Requirements map to specs, proposed modules, UI areas, and tests. |
| Testing strategy | Added | Local, mock, live Pixiv, DeepSeek, and visual tests are separated. |
| Architecture | Covered | Downloader-first backend/frontend module boundaries are specified. |
| Pixiv client | Added | Auth, DTOs, rate limits, retry, and mock-first rules are specified. |
| File storage | Added | V1 path layout, safe writes, existing-file behavior, and thumbnails are specified. |
| Error catalog | Added | Stable downloader/API/task error codes are specified. |
| Implementation plan | Added | Milestones prioritize the core download script before UI expansion. |
| Live Pixiv smoke | Passed | Single work `144920810` downloaded successfully to a temporary directory. |
| Current implementation phase | Anchored | Phase 3A Axum API wrapper, Phase 3B background queue, Phase 4A frontend scaffold, Phase 4B data API wiring, and Phase 4C frontend-configured single download are complete; next choice is secure gallery file/thumbnail serving. |

## Gaps to Resolve Before Implementation

### 1. Pixiv Access Method

Need to decide whether backend talks to Pixiv through:

- Direct Pixiv web/API calls implemented in Rust.
- A known Pixiv API crate/library if one is reliable.
- A small compatibility layer around existing Pixiv endpoints.

Impact:

- Determines request headers, rate limit handling, ranking/bookmark implementation, and how much mocking we need.

### 2. Credential Storage Detail

Current spec says secrets are local and masked. It does not yet decide whether secrets are:

- Stored directly in SQLite.
- Stored in a local config file.
- Stored through OS keychain later.

V1 can use SQLite, but this should be explicit before coding settings.

### 3. File Layout and Naming

Current recommendation is now defined in `file-storage.md`:

```text
{download_root}/originals/{pixiv_id}/{pixiv_id}_p{page_index}.{ext}
```

User confirmation is still useful before implementation because path layout becomes sticky once real files are downloaded.

### 4. Thumbnail Strategy

Current spec allows `thumbnail_path`, but does not define when/how thumbnails are generated.

Options:

- Use original image for MVP thumbnails.
- Generate local thumbnails during download.
- Defer thumbnail generation until first gallery request.

For performance, generated thumbnails are better, but they add image-processing dependencies.

### 5. Map-Based Search Definition

The PRD mentions map/heatmap search, and spec reserves `map_x/map_y`.

Still unclear:

- Are coordinates manually assigned?
- Generated from tags?
- Generated from AI/image embedding?
- Only a visual tag cloud for V1?

Recommendation: V1 starts as tag heatmap/manual coordinates, AI embeddings later.

### 6. R18/NSFW Semantics

Spec separates `r18` and `nsfw`, but Pixiv primarily exposes R18/R18G-style metadata while local NSFW may be user-defined.

Decision needed:

- Should `nsfw` be a local override only?
- Should R18G exist as a separate category?
- Should `ImageCategory` become `normal | r18 | r18g | nsfw_local`?

### 7. Top10 Caching

Current spec says refresh/cache, but not cache expiry.

Recommendation:

- Manual refresh always allowed.
- Daily cache keyed by date and ranking mode.
- Avoid refetching automatically more than once per day unless user clicks refresh.

### 8. Download Rate Limits and Courtesy Delays

Initial rate limits and retry rules are now defined in `pixiv-client.md`.

Still needs validation during live smoke tests because Pixiv behavior may vary by account/session/network.

### 9. Error Code Catalog

Stable error codes are now defined in `error-catalog.md`.

Implementation should use one typed backend error enum so task logs, API responses, and tests stay aligned.

### 10. Frontend Data Fetching Library

Spec does not pick frontend query/state tooling.

Reasonable default:

- Native `fetch` wrappers first.
- Add TanStack Query only if polling/cache complexity grows.

## Remaining Decisions Before Next Major Expansion

1. Decide whether R18G needs a separate category before schema migration is implemented.
2. Decide whether map search V1 is tag heatmap/manual coordinates or embedding-based.
3. Confirm whether local secrets should start in SQLite settings for V1, then later move to OS keychain.
4. Decide when thumbnail generation enters the plan: after DB dedupe, after gallery API, or after frontend gallery.

## Guided Questions for User

1. R18 细分是否需要把 R18G 独立出来，还是 V1 只做 `normal/r18/nsfw`？
2. 地图检索 V1 你希望是真的二维风格地图，还是先做 tag heatmap/标签空间的可视化？
3. 本地 secret V1 是否接受先存 SQLite 并 mask，后续再升级到系统 Keychain？

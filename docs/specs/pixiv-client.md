# Pixiv Client Specification

Requirements: `REQ-DL-001` through `REQ-DL-007`, `REQ-TASK-004`, `REQ-SEC-001`

## Goal

Provide a small, testable Pixiv access layer that can:

- Validate whether a user-provided `PHPSESSID` works.
- Fetch work metadata for a Pixiv ID.
- Resolve original image URLs for each page.
- Download image bytes with correct headers.
- Support current bookmark and author flows, plus future ranking, random, and tag-search flows.

## Client Interface

The downloader should depend on a trait, not a concrete HTTP implementation.

```rust
#[async_trait]
pub trait PixivClient {
    async fn check_auth(&self) -> Result<PixivAuthStatus, AppError>;
    async fn fetch_work(&self, pixiv_id: &str) -> Result<PixivWork, AppError>;
    async fn download_image(&self, url: &str) -> Result<Bytes, AppError>;
}
```

Current and later batch extensions:

```rust
async fn fetch_bookmarks(&self, limit: u32, r18_policy: R18Policy) -> Result<Vec<PixivWorkRef>, AppError>;
async fn fetch_author_works(&self, author_uid: &str, limit: u32) -> Result<Vec<PixivWorkRef>, AppError>;
async fn fetch_daily_top10(&self) -> Result<Vec<PixivWorkRef>, AppError>;
async fn search_by_tags(&self, tags: &[String], negative_tags: &[String], limit: u32, r18_policy: R18Policy) -> Result<Vec<PixivWorkRef>, AppError>;
```

## Required Request State

| State | Source | Notes |
| --- | --- | --- |
| `PHPSESSID` | settings/env | Required for live Pixiv access |
| `User-Agent` | backend config | Should be stable and explicit |
| `Referer` | Pixiv work page or `https://www.pixiv.net/` | Required for image download reliability |
| `Accept-Language` | backend config | Default can be `zh-CN,zh;q=0.9,en;q=0.8,ja;q=0.7` |
| `r18_policy` | request/settings | Determines filtering before download |

## Metadata DTOs

### `PixivWork`

| Field | Type |
| --- | --- |
| `pixiv_id` | string |
| `title` | string/null |
| `author_uid` | string/null |
| `author_name` | string/null |
| `tags` | string array |
| `category` | `ImageCategory` |
| `pages` | `PixivPage[]` |

### `PixivPage`

| Field | Type |
| --- | --- |
| `page_index` | integer |
| `original_url` | string |
| `width` | integer/null |
| `height` | integer/null |
| `extension` | string/null |

## R18 Filtering

- If request policy is `exclude`, the client/orchestrator must skip works categorized as R18 before downloading bytes.
- If policy is `only_r18`, non-R18 works are skipped.
- If policy is `include_blurred` or `include_visible`, downloads are allowed and visibility is handled by gallery/UI.

## Rate Limit and Retry

Initial defaults:

| Setting | Default |
| --- | --- |
| Global Pixiv request concurrency | `2` |
| Per-task image download concurrency | `2` |
| Retry count | `2` |
| Retry backoff | `500ms`, then `1500ms` |
| Delay between metadata requests | `300ms` |

Retry only:

- network timeout
- transient 5xx
- temporary rate-limit response

Do not retry:

- invalid cookie
- missing work
- R18 policy skip
- local validation errors

## Live Download Preconditions

Live tests require user-provided:

- `PIXIV_PHPSESSID`
- `PIXIV_TEST_WORK_ID`
- Optional `PIXIV_TEST_AUTHOR_UID`

The live smoke test should:

1. Use a temporary download directory.
2. Download at most one safe work by default.
3. Mask credentials in logs.
4. Print task/result summary without dumping raw cookies.

## Mock-First Rule

Before any live Pixiv call is used for development confidence, local tests must cover:

- Metadata fetch success.
- Image byte download success.
- Duplicate skip.
- R18 policy skip.
- Network failure.
- Filesystem failure.

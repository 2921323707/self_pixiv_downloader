# Live Smoke Tests

Live Pixiv tests are opt-in and must never store credentials in the repository.

## Current Smoke Result

Date: 2026-05-21

Command shape:

```text
PIXIV_TEST_WORK_ID=144920810
PIXIV_TEST_DOWNLOAD_DIR=/private/tmp/pixiv_platform_live_144920810
cargo run --bin live_single
```

Credential input:

- `PIXIV_PHPSESSID` was provided at runtime only.
- The cookie was not written to a project file.

Result:

- Status: `Saved`
- Pixiv ID: `144920810`
- Page index: `0`
- Title: `おでかけ`
- Tags parsed: `5`
- Pages parsed: `1`
- Saved file: `/private/tmp/pixiv_platform_live_144920810/originals/144920810/144920810_p0.png`
- File check: PNG, `1062 x 1500`, about `1.7M`

## Run Notes

Use a temporary download directory for live smoke tests unless you explicitly want to reuse an existing local library path.

The current E2E script uses the task-wrapped DB-aware downloader:

- `PIXIV_TEST_DOWNLOAD_DIR` defaults to a unique `/private/tmp/pixiv_platform_e2e_*` directory.
- `PIXIV_TEST_DB_PATH` defaults to `pixiv_platform.sqlite3` inside that directory.
- The first run verifies file write plus `tasks`, `task_items`, `task_logs`, `images`, `image_tags`, and `image_sources` indexing.
- The second run verifies duplicate skip from DB/file state.
- The script does not store Pixiv credentials and does not clean an existing user-provided directory.

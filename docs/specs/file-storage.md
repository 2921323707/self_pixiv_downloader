# File Storage Specification

Requirements: `REQ-DL-001`, `REQ-DL-006`, `REQ-IMG-001`, `REQ-IMG-003`, `REQ-SEC-002`

## Default Download Root

Default: `project:output`, resolved by the backend to the repository `output/` directory.

Tests and live smoke scripts should use a temporary directory unless the user explicitly chooses a persistent path.

## V1 File Layout

Recommended downloader-first layout:

```text
{download_root}/
  originals/
    {pixiv_id}/
      {pixiv_id}_p{page_index}.{ext}
  thumbnails/
    {pixiv_id}/
      {pixiv_id}_p{page_index}.webp
```

Rationale:

- Pixiv ID is the primary dedupe key.
- Multi-page works stay grouped.
- Source/category can change over time, so source/category should stay in SQLite rather than path identity.
- Gallery can derive all display state from DB, not folder names.

## Filename Rules

- `pixiv_id` must be sanitized as a safe path segment.
- `page_index` is zero-based.
- Extension comes from Pixiv original URL when trusted, otherwise content type.
- Unknown extension falls back to `.bin` and logs a warning.

## Safe Write Protocol

For every image:

1. Plan final path.
2. Check database dedupe.
3. Check whether final path already exists.
4. Write bytes to temp file in same directory.
5. Flush/sync best effort.
6. Rename temp file to final path.
7. Insert/update SQLite image metadata.

Temp pattern:

```text
.{pixiv_id}_p{page_index}.{uuid}.tmp
```

If DB insert fails after file write, keep the file and log `SQLITE_ERROR` with enough context for repair.

## Existing File Behavior

| Condition | Behavior |
| --- | --- |
| DB row exists and file exists | Skip download, record source history if needed |
| DB row exists but file missing | Redownload and repair `local_path` |
| File exists but DB row missing | Index existing file if path matches expected layout |
| Path collision with different identity | Fail with `FILESYSTEM_PATH_COLLISION` |

## Thumbnail Strategy

V1 recommendation:

- Start with original image serving for first implementation.
- Add thumbnail generation as second step after single download works.
- Store `thumbnail_path` when generated.

Reason:

- Reduces first downloader milestone risk.
- Avoids adding image-processing dependencies before network/file correctness is proven.

When thumbnails are added:

- Generate max width around `480px`.
- Prefer WebP if dependency support is reliable.
- Preserve aspect ratio.
- Failed thumbnail generation should not fail the original download task.

## Deletion Policy

V1 does not require destructive deletion.

Future deletion should separate:

- remove from library only
- delete local file
- delete original plus thumbnails

All destructive actions need explicit user confirmation in UI.

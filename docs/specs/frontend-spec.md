# Frontend Specification

Frontend stack: Next.js + CSS

The frontend is an image-first personal workstation for Pixiv anime illustrations. It should feel expressive and polished, but operational views must remain efficient and scannable.

Selected default visual direction: Demo B / `cyan-studio`.

## Route Plan

| Route | Page | Requirements |
| --- | --- | --- |
| `/` | Homepage | `REQ-UI-001`, `REQ-DL-004`, `REQ-DL-005` |
| `/download` | Download Center | `REQ-UI-002`, `REQ-DL-001` through `REQ-DL-005`, `REQ-AI-001` through `REQ-AI-004` |
| `/tasks` | Task Panel | `REQ-UI-003`, `REQ-TASK-001` through `REQ-TASK-005` |
| `/gallery` | Gallery | `REQ-UI-005`, `REQ-IMG-002` through `REQ-IMG-005`, `REQ-IMG-007` |
| `/settings` | Settings | `REQ-UI-004`, `REQ-CFG-001` through `REQ-CFG-006` |

## App Shell

Shared layout:

- Left sidebar or top navigation depending on viewport.
- Persistent active task indicator.
- Theme-aware background and surfaces.
- R18 visibility state visible in the shell when not `exclude`.
- Main content width optimized for image grids and task tables.

Primary navigation labels:

- Home
- Download
- Gallery
- Tasks
- Settings

## Homepage

Requirements: `REQ-UI-001`, `REQ-DL-004`, `REQ-DL-005`

Purpose: Fast entry into the user's local Pixiv world.

Sections:

- Daily Top10 carousel.
- Random Surprise action.
- Category entry chips/cards: Normal, R18/NSFW according to policy, Smart, Recent.
- Recent tasks compact strip.
- Recently downloaded image preview row.

States:

- Top10 empty: show refresh action.
- Top10 loading: carousel skeleton.
- Pixiv cookie missing: show settings action.
- R18 hidden: R18 entries absent or locked according to `r18_policy`.

Design notes:

- The carousel should use actual image thumbnails once available.
- Avoid marketing-style hero copy; the homepage is a dashboard.
- First viewport must show images or image placeholders, not only settings/forms.

## Download Center

Requirements: `REQ-UI-002`, `REQ-DL-001` through `REQ-DL-005`, `REQ-AI-001` through `REQ-AI-004`

Tabs:

- Single Work
- Bookmarks
- Author
- Top10
- Random
- Smart Retrieval

Shared controls:

- Quantity input where applicable.
- R18 policy segmented control.
- Destination/category selector if category editing is introduced.
- Submit button that creates a task and opens task status affordance.

Smart Retrieval flow:

1. User enters natural language prompt.
2. Frontend calls `POST /api/smart/parse`.
3. UI displays parsed tags, negative tags, recommended count, R18 policy.
4. User edits count/tags/policy.
5. User submits `POST /api/smart/download`.
6. UI links to task detail and gallery filter for the generated tags.

Validation:

- Pixiv IDs must be non-empty numeric strings unless backend supports broader formats.
- Quantity must be positive and no greater than configured max.
- Smart prompt must be non-empty.
- Missing cookie/API key should route user toward settings with a clear state marker.

## Task Panel

Requirements: `REQ-UI-003`, `REQ-TASK-001` through `REQ-TASK-005`

Views:

- Active queue.
- Completed history.
- Failed/errors.
- Task detail drawer/panel.

Task row fields:

- Type icon.
- Status badge.
- Progress bar.
- Current item.
- Created time.
- Finished time for terminal states.
- Error summary when present.

Task detail:

- Original request parameters.
- Progress numbers.
- Logs grouped by phase.
- Item-level failures.
- Open related gallery filter when images exist.

State display:

| Status | UI Treatment |
| --- | --- |
| `pending` | Quiet waiting state |
| `running` | Animated progress indicator |
| `completed` | Success badge |
| `completed_with_errors` | Warning badge with failure count |
| `failed` | Error badge and diagnostics |
| `cancelled` | Neutral stopped badge |

## Gallery

Requirements: `REQ-UI-005`, `REQ-IMG-002` through `REQ-IMG-005`, `REQ-IMG-007`

Primary layout:

- Masonry/waterfall image grid.
- Sticky filter toolbar.
- Optional right-side metadata/preview drawer on wide screens.
- Fullscreen preview modal for click-to-zoom.
- Selection mode for deleting unwanted local images from disk and SQLite.

Filters:

- Tags.
- Category.
- Author UID.
- Source.
- Date.
- R18 visibility.
- Map region/point selection.

Image card:

- Stable aspect-ratio placeholder.
- Thumbnail.
- Small source/category badges.
- Hover actions: zoom, metadata, edit tags/category.
- R18 blurred overlay when policy requires it.

Preview modal:

- Large image.
- Pixiv ID and page index.
- Title.
- Tags.
- Author UID.
- Source history.
- Local metadata.
- Next/previous navigation within current filter result.

Lazy loading:

- Initial page loads first batch only.
- Infinite scroll or explicit "load more" is acceptable.
- Cursor state should be reflected in URL or recoverable state where practical.

## Settings

Requirements: `REQ-UI-004`, `REQ-CFG-001` through `REQ-CFG-006`, `REQ-SEC-001`

Sections:

- Pixiv connection: PHPSESSID input, masked saved state, test button.
- DeepSeek: API key, base URL, test button.
- Local storage: download path and basic path status.
- Limits: max quantity per request.
- R18/NSFW policy: exclude, blurred, visible, only R18.
- Theme selection: five demo themes.

Secret behavior:

- Never display saved secret values in full.
- After saving, show masked status and last test result.
- Frontend should avoid keeping secrets in long-lived global state.

## Frontend State Model

| State | Location | Notes |
| --- | --- | --- |
| Current route | Next.js router | Standard route state |
| Theme | App root + backend settings | Applied before full render when possible |
| R18 visibility | Backend settings + page state | Must affect gallery/home/download controls |
| Active tasks | Query cache | Poll while active |
| Gallery filters | URL query + local state | Enables shareable/recoverable browsing |
| Smart parse result | Download page state | Reset when prompt changes significantly |
| Settings form drafts | Local component state | Secrets not global |

## Responsive Behavior

- Desktop: sidebar navigation, dense task tables, multi-column gallery.
- Tablet: top navigation or collapsed sidebar, medium-density gallery.
- Mobile: bottom/top navigation, single-column or two-column gallery depending on width, drawer-based filters.

## Accessibility and Interaction

- All icon-only buttons need tooltips and accessible labels.
- Keyboard navigation should work for modals, forms, and tabs.
- Status colors must be paired with text labels.
- R18 blur overlays should not be the only cue; category labels remain present.

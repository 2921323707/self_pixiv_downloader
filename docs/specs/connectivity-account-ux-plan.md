# Connectivity and Account UX Plan

Last updated: 2026-05-30

Branch: `codex/connectivity-account-ux`

## Implementation Status

### Completed in chunk 1

- Added migration `0002_pixiv_accounts` with local Pixiv account metadata and active-account tracking.
- Added backend `PixivAccountRepository`.
- Extended Pixiv client profile lookup to return `user_uid` and best-effort `user_name`.
- Extended `POST /api/settings/test/pixiv` so a verified cookie can record/update the active local Pixiv account.
- Added account APIs:
  - `GET /api/pixiv/accounts`
  - `POST /api/pixiv/accounts/active`
  - `DELETE /api/pixiv/accounts/{user_uid}`
- Extended the Tauri `refresh_pixiv_phpsessid` command response with `user_uid` and optional `user_name`.
- Added frontend API types/helpers for Pixiv account listing, activation, and deletion.
- Updated Settings Pixiv test/refresh copy to include resolved account identity.

### Completed in chunk 2

- Added `GET /api/runtime/readiness` with structured backend, Pixiv network, Pixiv account, and DeepSeek checks.
- Added Pixiv network probing with TUN-mode recovery guidance when Pixiv is unreachable.
- Added readiness tests for fully ready state and Pixiv network failure guidance.
- Added frontend readiness API types/helpers.
- Wired Home to load readiness status on entry and refresh.
- Added Home readiness UI with retry, Bind Pixiv, Settings, and current Pixiv account display.

### Completed in chunk 3

- Added Settings local Pixiv account list in the Pixiv category.
- Added Settings controls to refresh the local account list, switch the active account, and delete saved accounts.
- Kept Settings Pixiv test/refresh in sync with account list state after successful validation.
- Added a frontend account-change event so Home readiness can refresh after account mutations in the same window.
- Updated active account deletion to clear the runtime Pixiv cookie and active account metadata so readiness does not recreate the deleted account.
- Added backend regression coverage for deleting the active Pixiv account and returning Pixiv account readiness to `missing`.

Verified:

- `cd src/backend && cargo test`
- `cd src/frontend && npm run build`
- `cd tauri-app/src-tauri && cargo check`

### Next chunks

1. Manually validate the fresh DMG:
   - Home `Check Updates` opens `https://github.com/2921323707/self_pixiv_downloader/releases` in the system browser.
   - Home Configuration shows only Network, Pixiv binding, and DeepSeek API.
   - Network row shows connected/unreachable state and latency when available.
   - Download Path does not appear in Home Configuration.

### Completed in chunk 5

- Changed the Home update target from the repository root to `https://github.com/2921323707/self_pixiv_downloader/releases`.
- Added a restricted Tauri command, `open_external_url`, that only opens the approved Releases URL in the system browser.
- Added the matching desktop permission and capability entry for the update command.
- Added `latency_ms` to runtime readiness checks and surfaced Pixiv network latency in Home Configuration.
- Reduced Home Configuration to exactly three rows:
  - Network
  - Pixiv binding
  - DeepSeek API
- Removed DeepSeek key and Download Path from Home Configuration.
- Rebuilt the macOS DMG at `tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.2.0_aarch64.dmg`.

Verified:

- `cd src/frontend && npm run build`
- `cd src/backend && cargo test`
- `cd tauri-app/src-tauri && cargo check`
- `./tests/run_local.sh`
- `cd tauri-app && npm run build`
- `codesign --verify --deep --strict "tauri-app/src-tauri/target/release/bundle/macos/Pixiv Platform.app"`
- `hdiutil verify "tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.2.0_aarch64.dmg"`

### Completed in chunk 4

- Removed the standalone Home Runtime Readiness panel.
- Integrated Pixiv network, Pixiv account, and DeepSeek readiness into the existing Home Configuration panel.
- Changed the Home heading action from dashboard refresh to App software update, linking to `https://github.com/2921323707/self_pixiv_downloader`.
- Changed the top-right app shell chip from `Queue ready` to the current Pixiv UID, falling back to `UID: Not bound`.
- Added a lightweight top-level task poll so the Download nav icon animates while any recent task is `pending` or `running`.
- Made Home's automatic readiness check run only once per app/browser session; explicit account changes and binding still refresh readiness.
- Browser-checked Home at `http://127.0.0.1:3006` for UID display, update link, Configuration integration, Pixiv network/account rows, and absence of the standalone Runtime Readiness panel.
- Built a fresh macOS DMG at `tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.2.0_aarch64.dmg`.

Verified:

- `cd src/backend && cargo test`
- `cd src/frontend && npm run build`
- `./tests/run_local.sh`
- `cd tauri-app && npm run build`
- `codesign --verify --deep --strict "tauri-app/src-tauri/target/release/bundle/macos/Pixiv Platform.app"`
- `hdiutil verify "tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.2.0_aarch64.dmg"`

## Goal

Improve the first-run and daily-use feel of the desktop app by making runtime readiness visible before the user starts downloading:

- backend/local app health
- outbound network reachability to Pixiv
- real Pixiv cookie login validation
- DeepSeek configuration validation
- current Pixiv account identity on Home
- account switching for multiple locally bound Pixiv sessions

This is a local desktop UX improvement. It is not a cloud account system.

## Current Anchor

Already implemented:

- Tauri starts the local backend and waits for `/api/health` before creating the main window.
- Settings has manual `pixiv_cookie` and DeepSeek API key/base URL/model fields.
- Settings can test Pixiv through `POST /api/settings/test/pixiv`.
- Settings can test DeepSeek through `POST /api/settings/test/deepseek`.
- Tauri has `refresh_pixiv_phpsessid`, which opens a Pixiv login window, waits for a real `PHPSESSID`, validates it, saves it through Settings, and closes the login window.
- The Pixiv HTTP client can already resolve the current user UID from a verified cookie.

Gap:

- Readiness is scattered across Settings and not surfaced on app entry.
- Pixiv test responses do not expose account identity.
- Bound account state is represented only by the secret `pixiv_cookie`.
- There is no first-class multi-account switcher.
- Network failure guidance is generic; the desired desktop guidance is "enable TUN mode" when Pixiv cannot be reached.

## UX Flow

### App Entry Readiness

On the first Home load in an app/browser session, run a readiness check that reports:

| Check | Success | Failure guidance |
| --- | --- | --- |
| Local backend | API health responds | Show backend startup/log guidance |
| Pixiv network | Pixiv host responds without requiring login | "Network unreachable. Please enable TUN mode and retry." |
| Pixiv cookie | Current cookie validates against Pixiv | "Pixiv cookie is not connected." with a button to bind |
| DeepSeek | DeepSeek config test succeeds | "DeepSeek not configured/failed." with a Settings link |

The readiness status should be compact and action-first inside Home's existing Configuration panel, not a standalone Runtime Readiness page section. Returning to Home from Gallery, Tasks, Download, or Settings should reuse cached readiness state and normal dashboard data instead of automatically re-running real Pixiv/DeepSeek checks.

The Home heading action is reserved for App software updates and opens the GitHub Releases page in the system browser from desktop builds:

`https://github.com/2921323707/self_pixiv_downloader/releases`

Home Configuration is intentionally limited to three runtime rows:

- network connectivity and latency
- Pixiv binding status
- DeepSeek API connectivity

Storage paths and other static settings remain available in Settings and should not appear in Home Configuration.

### Pixiv Binding

1. User clicks "Bind Pixiv" or "Switch account".
2. Desktop opens the Pixiv login window.
3. App waits while the user finishes login.
4. Candidate `PHPSESSID` is validated against Pixiv.
5. After validation, app stores the cookie locally.
6. Login window closes.
7. Home shows "Bound" and the account identity.

### Account Identity

Current code can resolve the current user UID. The implementation should extend this into a Pixiv account profile:

- required: `user_uid`
- preferred: `user_name`
- optional: avatar/profile URL if available from a stable Pixiv response

If username parsing fails, the UI falls back to `Pixiv UID {user_uid}`. Binding should not fail solely because username is unavailable.

### Account Switching

Use local account profiles, not a server-side account model.

Recommended storage shape:

- keep the active runtime secret compatible with existing code via `pixiv_cookie`
- add public active account metadata such as `pixiv_active_account_uid`
- add a local profile store for multiple bound accounts with secret cookies

Implementation options:

1. SQLite table `pixiv_accounts`.
2. Settings JSON array with account metadata plus encrypted/secret cookie values.

Preferred: SQLite table, because it avoids stretching the key/value settings table and gives clean switch/delete/update semantics.

Switching an account sets the active `pixiv_cookie` to that account's cookie and updates public active account metadata. Existing download flows keep resolving the active cookie the same way they do today.

## API Shape

Add or extend endpoints:

- `GET /api/runtime/readiness`
  - returns structured statuses for backend, Pixiv network, Pixiv account, and DeepSeek.
- `POST /api/settings/test/pixiv`
  - extend response with `user_uid`, `user_name`, and `bound`.
- `GET /api/pixiv/accounts`
  - list locally bound accounts with non-secret metadata.
- `POST /api/pixiv/accounts/active`
  - switch active account by UID.
- `DELETE /api/pixiv/accounts/{user_uid}`
  - remove a bound account.

Tauri command:

- extend `refresh_pixiv_phpsessid` response with validated `user_uid` and optional `user_name`.

## Frontend Changes

- Home:
  - integrate runtime readiness into the existing Configuration panel.
  - show only network connectivity/latency, Pixiv binding, and DeepSeek API connectivity in Configuration.
  - show current Pixiv account UID when bound.
  - provide retry, bind, switch, and Settings actions.
  - replace the heading Refresh action with a GitHub Releases update link that opens in the system browser in desktop builds.
- App shell:
  - replace the top-right queue chip with `UID: {user_uid}` or `UID: Not bound`.
  - animate the Download nav icon when active tasks exist.
- Settings:
  - keep advanced raw configuration fields.
  - make Pixiv refresh/bind messaging consistent with Home.
  - show account list/switch controls if the account APIs are available.
- API client:
  - add typed readiness/account calls.

## Tests

Backend deterministic gates:

- readiness returns `pixiv_network` failure with TUN guidance when Pixiv probe fails.
- Pixiv test response includes account identity when the mock client supports it.
- account repository can upsert/list/switch/delete without exposing cookies in public responses.
- existing download tests keep using active `pixiv_cookie`.

Frontend gate:

- Home includes readiness/account anchors.
- Settings includes account switch/bind anchors.

Manual desktop validation:

- open app with no cookie: Home shows cookie not connected and bind action.
- click bind: login window opens, waits for real login, saves verified cookie, closes, Home shows bound account.
- disable proxy/TUN or block Pixiv: readiness suggests enabling TUN mode.
- save DeepSeek config: readiness reflects configured/failed state.

## Merge Plan

Develop on `codex/connectivity-account-ux`, push regularly, then open a PR back to `main`. Merge only after deterministic local tests pass and the desktop binding flow has at least one manual validation pass.

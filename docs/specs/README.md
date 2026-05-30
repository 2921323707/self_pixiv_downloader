# Pixiv Platform Specs

This folder contains implementation-oriented specifications for the Pixiv AI Downloader & Smart Retrieval Platform.

The project follows a spec-coding workflow:

1. Product requirements are converted into stable requirement IDs.
2. Domain state, data schema, API contracts, pages, and tests reference those IDs.
3. Implementation should keep code, tests, and migrations traceable back to the specs.

## Document Map

| Document | Purpose |
| --- | --- |
| `requirements.md` | Requirement IDs, acceptance criteria, and V1/V2 boundaries |
| `domain-model.md` | Core entities, enums, state machines, and invariants |
| `task-flow.md` | Async download queue lifecycle and failure behavior |
| `database-schema.md` | SQLite tables, indexes, and traceability fields |
| `api-contract.md` | Backend API surface for frontend/backend separation |
| `architecture.md` | Downloader-first backend/frontend architecture and module boundaries |
| `pixiv-client.md` | Pixiv access strategy, auth, rate limits, and download pipeline |
| `file-storage.md` | Local file layout, naming, temp files, and thumbnails |
| `error-catalog.md` | Stable error codes and user-facing recovery guidance |
| `implementation-plan.md` | Milestone checklist with requirement IDs and test gates |
| `frontend-spec.md` | Page structure, UX states, components, and route plan |
| `visual-theme.md` | Anime/Pixiv-oriented visual direction and theme system |
| `testing-strategy.md` | Unit, integration, visual, and live-download testing strategy |
| `spec-audit.md` | Completeness audit, gaps, risks, and guided questions |
| `traceability.md` | Requirement-to-implementation traceability matrix |
| `connectivity-account-ux-plan.md` | Startup readiness, Pixiv binding, and local account switching plan |

For a higher-level navigation map across all project documents, start with `docs/DOCUMENT_MAP.md`.

## Naming Conventions

Requirement IDs use prefixes by module:

| Prefix | Area |
| --- | --- |
| `REQ-DL-*` | Pixiv download |
| `REQ-AI-*` | Smart retrieval and DeepSeek |
| `REQ-IMG-*` | Image management and gallery |
| `REQ-TASK-*` | Async task queue |
| `REQ-CFG-*` | System settings |
| `REQ-UI-*` | Frontend pages and interactions |
| `REQ-THEME-*` | Visual theme and aesthetic system |
| `REQ-SEC-*` | Privacy, credential handling, and R18 visibility |

Implementation modules, database migrations, API handlers, and tests should reference these IDs in comments, test names, or adjacent documentation when useful.

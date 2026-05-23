#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FRONTEND_DIR="${ROOT_DIR}/src/frontend"

cd "${FRONTEND_DIR}"

echo "[frontend] home dashboard wiring"
grep -q "fetchTasks({ limit: 3 })" app/page.tsx
grep -q "fetchImages({ limit: 8, r18_visibility: \"exclude\" })" app/page.tsx
grep -q "fetchSettings()" app/page.tsx
grep -q "Home Dashboard" app/page.tsx
grep -q "home-image-banner" app/page.tsx
grep -q "Rust Core Driver" app/page.tsx
grep -q "Performance Watch" app/page.tsx
grep -q "Next Capability Slots" app/page.tsx
grep -q "selectBannerImages" app/page.tsx

echo "[frontend] phase 7B UI polish anchors"
grep -q "download-tool-tabs" app/download/page.tsx
grep -q "TagChipInput" app/download/page.tsx
grep -q "Smart tag chip editor" app/download/page.tsx
grep -q "response.text()" lib/api.ts
grep -q "image-detail-drawer" app/gallery/page.tsx
grep -q "closeDetailDrawer" app/gallery/page.tsx
grep -q "Task detail modal" app/tasks/page.tsx
grep -q "closeDetail" app/tasks/page.tsx
grep -q "fetchTasks({ limit: recentLimit })" app/tasks/page.tsx
grep -q "settingsCategories" app/settings/page.tsx

echo "[frontend] typecheck"
npm run lint

echo "[frontend] production build"
npm run build

echo "[frontend] done"

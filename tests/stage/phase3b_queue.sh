#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BACKEND_DIR="${ROOT_DIR}/src/backend"

cd "${BACKEND_DIR}"

echo "[phase3b] background queue API tests"
cargo test api::tests::req_dl_001_req_task_001_post_single_download_enqueues_and_returns_task_id --lib
cargo test api::tests::req_task_002_req_task_004_get_task_returns_items_and_logs --lib
cargo test api::tests::req_task_004_queued_single_download_preserves_failure_diagnostics --lib

echo "[phase3b] done"

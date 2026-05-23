#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BACKEND_DIR="${ROOT_DIR}/src/backend"

cd "${BACKEND_DIR}"

echo "[phase4b] gallery/settings/task-list repository tests"
cargo test images::tests::req_img_002_req_img_003_repository_lists_images_with_filters_and_cursor --lib
cargo test settings::tests::req_cfg_001_req_sec_001_settings_repository_saves_known_values_and_masks_secret --lib
cargo test tasks::tests::req_task_002_repository_lists_tasks_with_filters_and_cursor --lib

echo "[phase4b] gallery/settings/task-list API tests"
cargo test api::tests::req_img_002_req_ui_005_get_images_returns_gallery_metadata --lib
cargo test api::tests::req_task_002_req_ui_003_get_tasks_returns_task_list --lib
cargo test api::tests::req_cfg_001_req_sec_001_settings_api_lists_and_saves_masked_values --lib

echo "[phase4b] done"

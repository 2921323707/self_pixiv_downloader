#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BACKEND_DIR="${ROOT_DIR}/src/backend"

cd "${BACKEND_DIR}"

echo "[integration] SQLite migrations"
cargo test db::tests:: --lib

echo "[integration] image/settings repositories"
cargo test images::tests:: --lib
cargo test settings::tests:: --lib

echo "[integration] DB-aware downloader"
cargo test downloads::tests::req_dl_001_req_img_001_db_aware_first_download_indexes_file_tags_and_source --lib
cargo test downloads::tests::req_dl_006_db_duplicate_skip_avoids_image_download_and_records_source --lib
cargo test downloads::tests::req_dl_006_missing_file_repair_redownloads_and_refreshes_index --lib
cargo test downloads::tests::req_dl_006_existing_file_indexing_inserts_db_without_downloading_bytes --lib

echo "[integration] task lifecycle"
cargo test tasks::tests::req_task_001_single_download_task_completes_and_links_image --lib
cargo test tasks::tests::req_task_004_single_download_task_records_failure_diagnostics --lib

echo "[integration] done"

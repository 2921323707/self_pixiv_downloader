#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BACKEND_DIR="${ROOT_DIR}/src/backend"

cd "${BACKEND_DIR}"

echo "[phase4c] frontend-configured single download API tests"
cargo test api::tests::req_dl_001_req_cfg_002_single_download_uses_settings_cookie_and_download_root --lib
cargo test api::tests::req_dl_007_settings_backed_pixiv_cookie_is_required_before_enqueue --lib
cargo test api::tests::req_dl_007_settings_pixiv_test_uses_masked_cookie_without_download --lib

echo "[phase4c] settings validation tests"
cargo test settings::tests::req_cfg_001_settings_repository_rejects_unknown_or_invalid_values --lib

echo "[phase4c] done"

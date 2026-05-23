#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BACKEND_DIR="${ROOT_DIR}/src/backend"

cd "${BACKEND_DIR}"

echo "[phase6b] Pixiv smart tag search parser tests"
cargo test pixiv::http::tests::req_ai_002 --lib

echo "[phase6b] smart batch task worker tests"
cargo test tasks::tests::req_ai_002_smart_batch --lib

echo "[phase6b] smart download API tests"
cargo test api::tests::req_ai_002_post_smart_download --lib
cargo test api::tests::req_dl_007_post_smart_download --lib
cargo test api::tests::req_cfg_004_post_smart_download --lib

echo "[phase6b] done"

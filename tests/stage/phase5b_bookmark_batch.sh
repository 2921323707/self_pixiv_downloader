#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BACKEND_DIR="${ROOT_DIR}/src/backend"

cd "${BACKEND_DIR}"

echo "[phase5b] bookmark batch task tests"
cargo test tasks::tests::req_dl_002_bookmark_batch --lib

echo "[phase5b] bookmark batch API tests"
cargo test api::tests::req_dl_002_post_bookmark --lib
cargo test api::tests::req_cfg_004_post_bookmark --lib
cargo test api::tests::req_dl_007_post_bookmark --lib

echo "[phase5b] done"

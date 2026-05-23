#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BACKEND_DIR="${ROOT_DIR}/src/backend"

cd "${BACKEND_DIR}"

echo "[phase5a] author batch task tests"
cargo test tasks::tests::req_dl_003_author_batch --lib

echo "[phase5a] author batch API tests"
cargo test api::tests::req_dl_003_post_author --lib
cargo test api::tests::req_cfg_004_post_author --lib
cargo test api::tests::req_dl_007_post_author --lib

echo "[phase5a] done"

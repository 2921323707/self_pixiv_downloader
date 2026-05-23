#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BACKEND_DIR="${ROOT_DIR}/src/backend"

cd "${BACKEND_DIR}"

echo "[phase6a] DeepSeek smart parse unit tests"
cargo test ai::tests::req_ai_ --lib

echo "[phase6a] DeepSeek settings and API tests"
cargo test settings::tests::req_cfg_003 --lib
cargo test api::tests::req_cfg_003 --lib
cargo test api::tests::req_ai_001_post_smart_parse --lib
cargo test api::tests::req_ai_005_post_smart_parse --lib
cargo test api::tests::req_cfg_004_post_smart_parse --lib

echo "[phase6a] done"

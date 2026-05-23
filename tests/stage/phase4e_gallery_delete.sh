#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BACKEND_DIR="${ROOT_DIR}/src/backend"

cd "${BACKEND_DIR}"

echo "[phase4e] gallery delete repository tests"
cargo test images::tests::req_img_007 --lib

echo "[phase4e] gallery delete API tests"
cargo test api::tests::req_img_007 --lib

echo "[phase4e] done"

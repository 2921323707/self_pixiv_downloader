#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BACKEND_DIR="${ROOT_DIR}/src/backend"

cd "${BACKEND_DIR}"

echo "[phase4d] gallery file resolution tests"
cargo test images::tests::req_img_004 --lib

echo "[phase4d] gallery file API tests"
cargo test api::tests::req_img_004 --lib
cargo test api::tests::req_img_002_req_ui_005_get_images_returns_gallery_metadata --lib

echo "[phase4d] done"

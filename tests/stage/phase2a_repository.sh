#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BACKEND_DIR="${ROOT_DIR}/src/backend"

cd "${BACKEND_DIR}"

echo "[phase2a] SQLite migration tests"
cargo test db::tests:: --lib

echo "[phase2a] image repository tests"
cargo test images::tests:: --lib

echo "[phase2a] settings repository tests"
cargo test settings::tests:: --lib

echo "[phase2a] done"

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BACKEND_DIR="${ROOT_DIR}/src/backend"

if [[ ! -f "${BACKEND_DIR}/Cargo.toml" ]]; then
  echo "Backend Cargo.toml not found at ${BACKEND_DIR}" >&2
  exit 1
fi

cd "${BACKEND_DIR}"
cargo test

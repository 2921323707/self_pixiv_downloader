#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "[local] backend unit tests"
"${ROOT_DIR}/tests/unit/backend_unit.sh"

echo "[local] phase2a repository checks"
"${ROOT_DIR}/tests/stage/phase2a_repository.sh"

echo "[local] phase2c task checks"
"${ROOT_DIR}/tests/stage/phase2c_tasks.sh"

echo "[local] backend sqlite integration checks"
"${ROOT_DIR}/tests/integration/backend_sqlite.sh"

echo "[local] backend api smoke checks"
"${ROOT_DIR}/tests/smoke/backend_api.sh"

echo "[local] phase3b background queue checks"
"${ROOT_DIR}/tests/stage/phase3b_queue.sh"

echo "[local] phase4b data API checks"
"${ROOT_DIR}/tests/stage/phase4b_data_api.sh"

echo "[local] phase4c configured download checks"
"${ROOT_DIR}/tests/stage/phase4c_configured_download.sh"

echo "[local] phase4d gallery file API checks"
"${ROOT_DIR}/tests/stage/phase4d_gallery_file_api.sh"

echo "[local] phase4e gallery delete checks"
"${ROOT_DIR}/tests/stage/phase4e_gallery_delete.sh"

echo "[local] phase5a author batch checks"
"${ROOT_DIR}/tests/stage/phase5a_author_batch.sh"

echo "[local] phase5b bookmark batch checks"
"${ROOT_DIR}/tests/stage/phase5b_bookmark_batch.sh"

echo "[local] phase6a smart parse checks"
"${ROOT_DIR}/tests/stage/phase6a_smart_parse.sh"

echo "[local] phase6b smart download checks"
"${ROOT_DIR}/tests/stage/phase6b_smart_download.sh"

echo "[local] frontend scaffold checks"
"${ROOT_DIR}/tests/stage/frontend_scaffold.sh"

echo "[local] done"

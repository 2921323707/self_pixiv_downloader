#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BACKEND_DIR="${ROOT_DIR}/src/backend"

if [[ -z "${PIXIV_PHPSESSID:-}" ]]; then
  echo "PIXIV_PHPSESSID is required for live e2e tests." >&2
  echo "Provide it as an environment variable; do not write it to repository files." >&2
  exit 2
fi

export PIXIV_TEST_WORK_ID="${PIXIV_TEST_WORK_ID:-144920810}"
run_id="$(date +%Y%m%d%H%M%S)_$$"
export PIXIV_TEST_DOWNLOAD_DIR="${PIXIV_TEST_DOWNLOAD_DIR:-/private/tmp/pixiv_platform_e2e_${PIXIV_TEST_WORK_ID}_${run_id}}"
export PIXIV_TEST_DB_PATH="${PIXIV_TEST_DB_PATH:-${PIXIV_TEST_DOWNLOAD_DIR}/pixiv_platform.sqlite3}"

mkdir -p "${PIXIV_TEST_DOWNLOAD_DIR}"

cd "${BACKEND_DIR}"

echo "[e2e] live single download"
echo "[e2e] work_id=${PIXIV_TEST_WORK_ID}"
echo "[e2e] download_dir=${PIXIV_TEST_DOWNLOAD_DIR}"
echo "[e2e] db_path=${PIXIV_TEST_DB_PATH}"

first_output="$(cargo run --quiet --bin live_single)"
echo "${first_output}"

first_status="$(printf '%s\n' "${first_output}" | awk -F= '/^status=/{print $2}')"
first_task_status="$(printf '%s\n' "${first_output}" | awk -F= '/^task_status=/{print $2}')"
local_path="$(printf '%s\n' "${first_output}" | awk -F= '/^local_path=/{print $2}')"
db_image_id="$(printf '%s\n' "${first_output}" | awk -F= '/^db_image_id=/{print $2}')"
db_sources="$(printf '%s\n' "${first_output}" | awk -F= '/^db_sources=/{print $2}')"
task_items="$(printf '%s\n' "${first_output}" | awk -F= '/^task_items=/{print $2}')"
task_logs="$(printf '%s\n' "${first_output}" | awk -F= '/^task_logs=/{print $2}')"

if [[ "${first_status}" != "Saved" && "${first_status}" != "SkippedDuplicate" ]]; then
  echo "Unexpected live download status: ${first_status}" >&2
  exit 1
fi

if [[ "${first_task_status}" != "completed" ]]; then
  echo "Unexpected task status: ${first_task_status}" >&2
  exit 1
fi

if [[ -z "${local_path}" || ! -f "${local_path}" ]]; then
  echo "Downloaded file was not found: ${local_path}" >&2
  exit 1
fi

if [[ ! -s "${local_path}" ]]; then
  echo "Downloaded file is empty: ${local_path}" >&2
  exit 1
fi

if [[ -z "${db_image_id}" ]]; then
  echo "Downloaded image was not indexed in SQLite." >&2
  exit 1
fi

if [[ -z "${db_sources}" || "${db_sources}" -lt 1 ]]; then
  echo "Downloaded image source history was not recorded." >&2
  exit 1
fi

if [[ -z "${task_items}" || "${task_items}" -lt 1 ]]; then
  echo "Task item was not recorded." >&2
  exit 1
fi

if [[ -z "${task_logs}" || "${task_logs}" -lt 1 ]]; then
  echo "Task logs were not recorded." >&2
  exit 1
fi

echo "[e2e] second run should skip from DB state"
second_output="$(cargo run --quiet --bin live_single)"
echo "${second_output}"

second_status="$(printf '%s\n' "${second_output}" | awk -F= '/^status=/{print $2}')"
second_task_status="$(printf '%s\n' "${second_output}" | awk -F= '/^task_status=/{print $2}')"
second_local_path="$(printf '%s\n' "${second_output}" | awk -F= '/^local_path=/{print $2}')"
second_db_image_id="$(printf '%s\n' "${second_output}" | awk -F= '/^db_image_id=/{print $2}')"

if [[ "${second_status}" != "SkippedDuplicate" ]]; then
  echo "Expected second run to skip duplicate from DB state, got: ${second_status}" >&2
  exit 1
fi

if [[ "${second_task_status}" != "completed" ]]; then
  echo "Unexpected second task status: ${second_task_status}" >&2
  exit 1
fi

if [[ "${second_local_path}" != "${local_path}" ]]; then
  echo "Second run returned a different local path: ${second_local_path}" >&2
  exit 1
fi

if [[ "${second_db_image_id}" != "${db_image_id}" ]]; then
  echo "Second run returned a different DB image id: ${second_db_image_id}" >&2
  exit 1
fi

echo "[e2e] verified_file=${local_path}"
echo "[e2e] verified_db_image=${db_image_id}"

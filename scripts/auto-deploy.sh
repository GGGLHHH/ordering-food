#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEFAULT_REPO_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

REPO_DIR="${REPO_DIR:-${DEFAULT_REPO_DIR}}"
REMOTE="${REMOTE:-origin}"
CURRENT_BRANCH="$(git -C "${REPO_DIR}" rev-parse --abbrev-ref HEAD 2>/dev/null || echo main)"
BRANCH="${BRANCH:-${CURRENT_BRANCH}}"
HEALTH_TIMEOUT="${HEALTH_TIMEOUT:-180}"
COMPOSE_FILE_PATH="${COMPOSE_FILE_PATH:-${REPO_DIR}/compose.prod.yml}"
AUTO_DEPLOY_DIR="${AUTO_DEPLOY_DIR:-${REPO_DIR}/.git/auto-deploy}"
STATE_FILE="${STATE_FILE:-${AUTO_DEPLOY_DIR}/state}"
LOCK_DIR="${LOCK_DIR:-${AUTO_DEPLOY_DIR}/lock}"
DEPLOY_SERVICES=(server frontend nginx)

log() {
  printf '[%s] %s\n' "$(date '+%Y-%m-%d %H:%M:%S')" "$*"
}

fail() {
  log "ERROR: $*"
  exit 1
}

require_command() {
  local command_name="$1"
  command -v "${command_name}" >/dev/null 2>&1 || fail "Missing required command: ${command_name}"
}

release_lock() {
  if [ -d "${LOCK_DIR}" ]; then
    rm -rf "${LOCK_DIR}"
  fi
}

acquire_lock() {
  local lock_pid=''

  mkdir -p "$(dirname "${LOCK_DIR}")"

  if mkdir "${LOCK_DIR}" 2>/dev/null; then
    printf '%s\n' "$$" > "${LOCK_DIR}/pid"
    trap release_lock EXIT INT TERM
    return
  fi

  if [ -f "${LOCK_DIR}/pid" ]; then
    lock_pid="$(tr -d '[:space:]' < "${LOCK_DIR}/pid")"
    if [ -n "${lock_pid}" ] && ! kill -0 "${lock_pid}" 2>/dev/null; then
      log "Removing stale lock owned by pid ${lock_pid}"
      rm -rf "${LOCK_DIR}"
      mkdir "${LOCK_DIR}"
      printf '%s\n' "$$" > "${LOCK_DIR}/pid"
      trap release_lock EXIT INT TERM
      return
    fi
  fi

  fail "Another auto-deploy process is already running"
}

ensure_repo_ready() {
  require_command git
  require_command bash
  require_command docker

  [ -d "${REPO_DIR}/.git" ] || fail "REPO_DIR is not a git repository: ${REPO_DIR}"
  [ -f "${COMPOSE_FILE_PATH}" ] || fail "Compose file not found: ${COMPOSE_FILE_PATH}"

  if ! git -C "${REPO_DIR}" remote get-url "${REMOTE}" >/dev/null 2>&1; then
    fail "Git remote '${REMOTE}' not found in ${REPO_DIR}"
  fi

  case "${HEALTH_TIMEOUT}" in
    ''|*[!0-9]*) fail "HEALTH_TIMEOUT must be an integer number of seconds" ;;
  esac
}

is_worktree_clean() {
  [ -z "$(git -C "${REPO_DIR}" status --porcelain)" ]
}

fetch_remote() {
  git -C "${REPO_DIR}" fetch --quiet "${REMOTE}" "${BRANCH}"
}

remote_commit() {
  git -C "${REPO_DIR}" rev-parse "${REMOTE}/${BRANCH}"
}

current_commit() {
  git -C "${REPO_DIR}" rev-parse HEAD
}

read_state() {
  if [ -f "${STATE_FILE}" ]; then
    tr -d '[:space:]' < "${STATE_FILE}"
  fi
}

write_state() {
  mkdir -p "$(dirname "${STATE_FILE}")"
  printf '%s\n' "$1" > "${STATE_FILE}"
}

checkout_target_branch() {
  if git -C "${REPO_DIR}" show-ref --verify --quiet "refs/heads/${BRANCH}"; then
    git -C "${REPO_DIR}" checkout --quiet "${BRANCH}"
    git -C "${REPO_DIR}" merge --ff-only "${REMOTE}/${BRANCH}"
    return
  fi

  git -C "${REPO_DIR}" checkout --quiet -b "${BRANCH}" --track "${REMOTE}/${BRANCH}"
}

deploy_target() {
  local target_commit="$1"
  local previous_commit="$2"

  if ! is_worktree_clean; then
    log "Worktree is dirty, skipping deploy to avoid overwriting local changes"
    return 0
  fi

  if [ "$(git -C "${REPO_DIR}" rev-parse --abbrev-ref HEAD)" != "${BRANCH}" ] || [ "$(current_commit)" != "${target_commit}" ]; then
    checkout_target_branch
  fi

  log "Deploying ${previous_commit} -> ${target_commit}"
  (
    cd "${REPO_DIR}"
    docker compose -f "${COMPOSE_FILE_PATH}" build --no-cache "${DEPLOY_SERVICES[@]}"
    docker compose -f "${COMPOSE_FILE_PATH}" up -d --no-build --wait --wait-timeout "${HEALTH_TIMEOUT}" "${DEPLOY_SERVICES[@]}"
  )
  write_state "${target_commit}"
  log "Deploy completed for commit ${target_commit}"
}

main() {
  local deployed_commit=''
  local repo_commit=''
  local target_commit=''

  acquire_lock
  ensure_repo_ready

  log "Checking ${REMOTE}/${BRANCH} in ${REPO_DIR}"

  fetch_remote
  target_commit="$(remote_commit)"
  repo_commit="$(current_commit)"
  deployed_commit="$(read_state)"

  if [ -n "${deployed_commit}" ] && [ "${deployed_commit}" = "${target_commit}" ] && [ "${repo_commit}" = "${target_commit}" ]; then
    log "No update detected on ${REMOTE}/${BRANCH}; current commit: ${target_commit}"
    return 0
  fi

  deploy_target "${target_commit}" "${deployed_commit:-<none>}"
  log 'Single-run deploy check finished'
}

main "$@"

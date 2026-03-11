#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEFAULT_REPO_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

REPO_DIR="${REPO_DIR:-${DEFAULT_REPO_DIR}}"
REMOTE="${REMOTE:-origin}"
CURRENT_BRANCH="$(git -C "${REPO_DIR}" rev-parse --abbrev-ref HEAD 2>/dev/null || echo main)"
BRANCH="${BRANCH:-${CURRENT_BRANCH}}"
POLL_INTERVAL="${POLL_INTERVAL:-60}"
DEPLOY_COMMAND="${DEPLOY_COMMAND:-docker compose up -d --build server frontend nginx}"
HEALTH_CONTAINERS="${HEALTH_CONTAINERS-ordering-food-server ordering-food-frontend ordering-food-nginx}"
HEALTH_TIMEOUT="${HEALTH_TIMEOUT:-180}"
RUN_ONCE="${RUN_ONCE:-0}"
STATE_FILE="${STATE_FILE:-${REPO_DIR}/.auto-deploy-state}"
LOCK_FILE="${LOCK_FILE:-${REPO_DIR}/.auto-deploy.lock}"

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

acquire_lock() {
  exec 9>"${LOCK_FILE}"

  if command -v flock >/dev/null 2>&1; then
    flock -n 9 || fail "Another auto-deploy process is already running"
    return
  fi

  log "WARN: 'flock' not found, continuing without process lock"
}

ensure_repo_ready() {
  require_command git
  require_command bash
  require_command docker

  [ -d "${REPO_DIR}/.git" ] || fail "REPO_DIR is not a git repository: ${REPO_DIR}"
  [ -f "${REPO_DIR}/compose.yml" ] || fail "compose.yml not found under ${REPO_DIR}"

  if ! git -C "${REPO_DIR}" remote get-url "${REMOTE}" >/dev/null 2>&1; then
    fail "Git remote '${REMOTE}' not found in ${REPO_DIR}"
  fi

  case "${POLL_INTERVAL}" in
    ''|*[!0-9]*) fail "POLL_INTERVAL must be an integer number of seconds" ;;
  esac

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

wait_for_health() {
  if [ -z "${HEALTH_CONTAINERS}" ]; then
    log "Skipping health checks because HEALTH_CONTAINERS is empty"
    return 0
  fi

  local containers=()
  local container_name=''
  local status=''
  local all_healthy=1
  local deadline=$((SECONDS + HEALTH_TIMEOUT))

  read -r -a containers <<< "${HEALTH_CONTAINERS}"

  for container_name in "${containers[@]}"; do
    docker inspect "${container_name}" >/dev/null 2>&1 || fail "Container not found after deploy: ${container_name}"
  done

  while [ "${SECONDS}" -lt "${deadline}" ]; do
    all_healthy=1

    for container_name in "${containers[@]}"; do
      status="$(docker inspect --format '{{if .State.Health}}{{.State.Health.Status}}{{else}}{{.State.Status}}{{end}}' "${container_name}" 2>/dev/null || echo missing)"

      case "${status}" in
        healthy|running) ;;
        *)
          all_healthy=0
          log "Waiting for ${container_name} to become ready, current status: ${status}"
          ;;
      esac
    done

    if [ "${all_healthy}" -eq 1 ]; then
      log "All containers are healthy: ${HEALTH_CONTAINERS}"
      return 0
    fi

    sleep 5
  done

  fail "Timed out waiting for containers to become healthy"
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
  bash -lc "cd \"${REPO_DIR}\" && ${DEPLOY_COMMAND}"
  wait_for_health
  write_state "${target_commit}"
  log "Deploy completed for commit ${target_commit}"
}

run_iteration() {
  local deployed_commit=''
  local repo_commit=''
  local target_commit=''

  fetch_remote
  target_commit="$(remote_commit)"
  repo_commit="$(current_commit)"
  deployed_commit="$(read_state)"

  if [ -n "${deployed_commit}" ] && [ "${deployed_commit}" = "${target_commit}" ] && [ "${repo_commit}" = "${target_commit}" ]; then
    log "No update detected on ${REMOTE}/${BRANCH}; current commit: ${target_commit}"
    return 0
  fi

  deploy_target "${target_commit}" "${deployed_commit:-<none>}"
}

main() {
  acquire_lock
  ensure_repo_ready

  log "Watching ${REMOTE}/${BRANCH} in ${REPO_DIR} every ${POLL_INTERVAL}s"

  while true; do
    if ! run_iteration; then
      log "Iteration failed; will retry on the next cycle"
    fi

    if [ "${RUN_ONCE}" = '1' ]; then
      log 'RUN_ONCE=1, exiting after a single iteration'
      break
    fi

    sleep "${POLL_INTERVAL}"
  done
}

main "$@"

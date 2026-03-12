#!/usr/bin/env bash

set -euo pipefail

METHOD="GET"
JSON_BODY=""
JSON_FILE=""
RAW_BODY=""
RAW_BODY_FILE=""
BASE_URL=""
HOST=""
PORT=""
SCHEME="http"
TIMEOUT="10"
READY_PATH="/health/ready"
SKIP_READY_CHECK=0
FAIL_STATUS=0
SHOW_COOKIES=0
COOKIE_JAR=""
COOKIE_JAR_OWNED=0
BEARER_TOKEN=""
ACCESS_TOKEN=""
REFRESH_TOKEN=""
LOGIN_EMAIL=""
LOGIN_IDENTIFIER=""
LOGIN_PASSWORD=""
LOGIN_IDENTITY_TYPE="email"
LOGIN_ENDPOINT="/api/auth/login"
PATH_ARG=""

HEADERS=()
QUERY_ITEMS=()
EXTRA_COOKIES=()

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cleanup() {
  if [[ "$COOKIE_JAR_OWNED" -eq 1 && -n "$COOKIE_JAR" && -f "$COOKIE_JAR" ]]; then
    rm -f "$COOKIE_JAR"
  fi
  [[ -n "${TMP_DIR:-}" && -d "${TMP_DIR:-}" ]] && rm -rf "$TMP_DIR"
}

trap cleanup EXIT

usage() {
  cat <<'EOF'
Usage:
  request_api.sh <path> [options]

Options:
  --method <METHOD>              HTTP method, default GET
  --json <JSON>                  Inline JSON request body
  --json-file <FILE>             JSON request body file
  --body <TEXT>                  Inline raw request body
  --body-file <FILE>             Raw request body file
  --header <Name:Value>          Extra request header, repeatable
  --query <key=value>            Extra query item, repeatable
  --base-url <URL>               Override base URL
  --host <HOST>                  Override APP__HOST from .env
  --port <PORT>                  Override APP__PORT from .env
  --scheme <SCHEME>              URL scheme when base URL is not set
  --timeout <SECONDS>            Request timeout, default 10
  --ready-path <PATH>            Ready check path, default /health/ready
  --skip-ready-check             Skip ready check
  --cookie-jar <FILE>            Reuse cookies from a jar file
  --show-cookies                 Print cookies after the final request
  --fail-status                  Exit non-zero when final response is not 2xx
  --bearer-token <TOKEN>         Send Authorization header
  --access-token <TOKEN>         Send access_token cookie
  --refresh-token <TOKEN>        Send refresh_token cookie
  --cookie <name=value>          Extra cookie, repeatable
  --login-email <EMAIL>          Login with email first
  --login-identifier <VALUE>     Login identifier when not using email
  --login-password <PASSWORD>    Login password
  --login-identity-type <TYPE>   identity_type for login, default email
  --login-endpoint <PATH>        Login endpoint, default /api/auth/login
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --method)
      METHOD="$2"
      shift 2
      ;;
    --json)
      JSON_BODY="$2"
      shift 2
      ;;
    --json-file)
      JSON_FILE="$2"
      shift 2
      ;;
    --body)
      RAW_BODY="$2"
      shift 2
      ;;
    --body-file)
      RAW_BODY_FILE="$2"
      shift 2
      ;;
    --header)
      HEADERS+=("$2")
      shift 2
      ;;
    --query)
      QUERY_ITEMS+=("$2")
      shift 2
      ;;
    --base-url)
      BASE_URL="$2"
      shift 2
      ;;
    --host)
      HOST="$2"
      shift 2
      ;;
    --port)
      PORT="$2"
      shift 2
      ;;
    --scheme)
      SCHEME="$2"
      shift 2
      ;;
    --timeout)
      TIMEOUT="$2"
      shift 2
      ;;
    --ready-path)
      READY_PATH="$2"
      shift 2
      ;;
    --skip-ready-check)
      SKIP_READY_CHECK=1
      shift
      ;;
    --cookie-jar)
      COOKIE_JAR="$2"
      shift 2
      ;;
    --show-cookies)
      SHOW_COOKIES=1
      shift
      ;;
    --fail-status)
      FAIL_STATUS=1
      shift
      ;;
    --bearer-token)
      BEARER_TOKEN="$2"
      shift 2
      ;;
    --access-token)
      ACCESS_TOKEN="$2"
      shift 2
      ;;
    --refresh-token)
      REFRESH_TOKEN="$2"
      shift 2
      ;;
    --cookie)
      EXTRA_COOKIES+=("$2")
      shift 2
      ;;
    --login-email)
      LOGIN_EMAIL="$2"
      shift 2
      ;;
    --login-identifier)
      LOGIN_IDENTIFIER="$2"
      shift 2
      ;;
    --login-password)
      LOGIN_PASSWORD="$2"
      shift 2
      ;;
    --login-identity-type)
      LOGIN_IDENTITY_TYPE="$2"
      shift 2
      ;;
    --login-endpoint)
      LOGIN_ENDPOINT="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --*)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
    *)
      if [[ -n "$PATH_ARG" ]]; then
        echo "Only one request path is supported." >&2
        exit 1
      fi
      PATH_ARG="$1"
      shift
      ;;
  esac
done

if [[ -z "$PATH_ARG" ]]; then
  usage >&2
  exit 1
fi

payload_sources=0
[[ -n "$JSON_BODY" ]] && payload_sources=$((payload_sources + 1))
[[ -n "$JSON_FILE" ]] && payload_sources=$((payload_sources + 1))
[[ -n "$RAW_BODY" ]] && payload_sources=$((payload_sources + 1))
[[ -n "$RAW_BODY_FILE" ]] && payload_sources=$((payload_sources + 1))

if [[ "$payload_sources" -gt 1 ]]; then
  echo "Use only one of --json, --json-file, --body, or --body-file." >&2
  exit 1
fi

if [[ -n "$LOGIN_EMAIL" && -n "$LOGIN_IDENTIFIER" ]]; then
  echo "Use only one of --login-email or --login-identifier." >&2
  exit 1
fi

find_repo_root() {
  local current
  current="$(pwd)"
  while [[ "$current" != "/" ]]; do
    if [[ -f "$current/.env" ]]; then
      printf '%s\n' "$current"
      return 0
    fi
    current="$(dirname "$current")"
  done

  current="$SCRIPT_DIR"
  while [[ "$current" != "/" ]]; do
    if [[ -f "$current/.env" ]]; then
      printf '%s\n' "$current"
      return 0
    fi
    current="$(dirname "$current")"
  done

  return 1
}

REPO_ROOT="$(find_repo_root)" || {
  echo "Could not find the repository root .env file." >&2
  exit 1
}

ENV_FILE="$REPO_ROOT/.env"

read_env_value() {
  local key="$1"
  awk -F= -v key="$key" '
    $1 == key {
      sub(/^[^=]+= */, "", $0)
      print $0
      exit
    }
  ' "$ENV_FILE"
}

if [[ -z "$BASE_URL" ]]; then
  [[ -z "$HOST" ]] && HOST="$(read_env_value "APP__HOST")"
  [[ -z "$PORT" ]] && PORT="$(read_env_value "APP__PORT")"
  [[ -z "$HOST" ]] && HOST="127.0.0.1"
  [[ -z "$PORT" ]] && PORT="8080"
  BASE_URL="${SCHEME}://${HOST}:${PORT}"
fi

TMP_DIR="$(mktemp -d)"

if [[ -z "$COOKIE_JAR" && ( -n "$LOGIN_EMAIL" || -n "$LOGIN_IDENTIFIER" || "$SHOW_COOKIES" -eq 1 ) ]]; then
  COOKIE_JAR="$TMP_DIR/cookies.jar"
  COOKIE_JAR_OWNED=1
fi

if [[ -n "$COOKIE_JAR" ]]; then
  mkdir -p "$(dirname "$COOKIE_JAR")"
  touch "$COOKIE_JAR"
fi

is_success_status() {
  local status="$1"
  [[ "$status" -ge 200 && "$status" -lt 300 ]]
}

join_cookies() {
  local output=""
  local first=1
  local item
  for item in "$@"; do
    [[ -z "$item" ]] && continue
    if [[ "$first" -eq 1 ]]; then
      output="$item"
      first=0
    else
      output="${output}; ${item}"
    fi
  done
  printf '%s' "$output"
}

build_url() {
  local path="$1"
  local query_string=""
  local separator="?"
  local item

  if [[ "$path" =~ ^https?:// ]]; then
    printf '%s' "$path"
    return 0
  fi

  if [[ "${#QUERY_ITEMS[@]}" -gt 0 ]]; then
    local first=1
    for item in "${QUERY_ITEMS[@]}"; do
      if [[ "$first" -eq 1 ]]; then
        query_string="$item"
        first=0
      else
        query_string="${query_string}&${item}"
      fi
    done
  fi

  if [[ "$path" == /* ]]; then
    [[ "$path" == *\?* ]] && separator="&"
    if [[ -n "$query_string" ]]; then
      printf '%s%s%s%s' "$BASE_URL" "$path" "$separator" "$query_string"
    else
      printf '%s%s' "$BASE_URL" "$path"
    fi
  else
    [[ "$path" == *\?* ]] && separator="&"
    if [[ -n "$query_string" ]]; then
      printf '%s/%s%s%s' "$BASE_URL" "$path" "$separator" "$query_string"
    else
      printf '%s/%s' "$BASE_URL" "$path"
    fi
  fi
}

json_escape() {
  printf '%s' "$1" | sed \
    -e 's/\\/\\\\/g' \
    -e 's/"/\\"/g'
}

run_curl_request() {
  local method="$1"
  local url="$2"
  local body_mode="$3"
  local body_value="$4"

  LAST_HEADERS_FILE="$TMP_DIR/headers-$(date +%s)-$$.txt"
  LAST_BODY_FILE="$TMP_DIR/body-$(date +%s)-$$.txt"

  local -a curl_args
  curl_args=(-sS -D "$LAST_HEADERS_FILE" -o "$LAST_BODY_FILE" -X "$method" --max-time "$TIMEOUT")

  if [[ -n "$COOKIE_JAR" ]]; then
    curl_args+=(-b "$COOKIE_JAR" -c "$COOKIE_JAR")
  fi

  local inline_cookies=()
  [[ -n "$ACCESS_TOKEN" ]] && inline_cookies+=("access_token=$ACCESS_TOKEN")
  [[ -n "$REFRESH_TOKEN" ]] && inline_cookies+=("refresh_token=$REFRESH_TOKEN")
  if [[ "${#EXTRA_COOKIES[@]}" -gt 0 ]]; then
    inline_cookies+=("${EXTRA_COOKIES[@]}")
  fi
  if [[ "${#inline_cookies[@]}" -gt 0 ]]; then
    curl_args+=(-b "$(join_cookies "${inline_cookies[@]}")")
  fi

  local header
  for header in "${HEADERS[@]}"; do
    curl_args+=(-H "$header")
  done
  if [[ -n "$BEARER_TOKEN" ]]; then
    curl_args+=(-H "Authorization: Bearer $BEARER_TOKEN")
  fi

  case "$body_mode" in
    json)
      curl_args+=(-H "Content-Type: application/json" --data-binary "$body_value")
      ;;
    json_file)
      curl_args+=(-H "Content-Type: application/json" --data-binary "@$body_value")
      ;;
    raw)
      curl_args+=(--data-binary "$body_value")
      ;;
    raw_file)
      curl_args+=(--data-binary "@$body_value")
      ;;
  esac

  LAST_STATUS="$(curl "${curl_args[@]}" -w '%{http_code}' "$url")" || {
    echo "Failed to reach $url. Make sure the local API server is running." >&2
    exit 1
  }
}

ready_check() {
  local ready_url
  local saved_query_items=("${QUERY_ITEMS[@]}")
  ready_url="$(build_url "$READY_PATH")"
  QUERY_ITEMS=()
  run_curl_request "GET" "$ready_url" "" ""
  QUERY_ITEMS=("${saved_query_items[@]}")
  if [[ "$LAST_STATUS" != "200" ]]; then
    echo "Local API is not ready." >&2
    echo "Checked: $ready_url" >&2
    echo "Status: $LAST_STATUS" >&2
    echo "Body:" >&2
    cat "$LAST_BODY_FILE" >&2
    echo >&2
    echo "Start the server with \`make run\` or \`make dev\` and try again." >&2
    exit 1
  fi
  READY_STATUS="$LAST_STATUS"
  READY_URL="$ready_url"
}

if [[ "$SKIP_READY_CHECK" -eq 0 ]]; then
  ready_check
else
  READY_STATUS=""
  READY_URL=""
fi

if [[ -n "$LOGIN_EMAIL" || -n "$LOGIN_IDENTIFIER" ]]; then
  if [[ -z "$LOGIN_PASSWORD" ]]; then
    echo "--login-password is required when login credentials are provided." >&2
    exit 1
  fi

  login_identifier="$LOGIN_IDENTIFIER"
  [[ -z "$login_identifier" ]] && login_identifier="$LOGIN_EMAIL"
  login_payload="$(printf '{"identity_type":"%s","identifier":"%s","password":"%s"}' \
    "$(json_escape "$LOGIN_IDENTITY_TYPE")" \
    "$(json_escape "$login_identifier")" \
    "$(json_escape "$LOGIN_PASSWORD")")"
  local_saved_query_items=("${QUERY_ITEMS[@]}")
  QUERY_ITEMS=()
  LOGIN_URL="$(build_url "$LOGIN_ENDPOINT")"
  run_curl_request "POST" "$LOGIN_URL" "json" "$login_payload"
  QUERY_ITEMS=("${local_saved_query_items[@]}")
  if ! is_success_status "$LAST_STATUS"; then
    echo "Login failed." >&2
    echo "URL: $LOGIN_URL" >&2
    echo "Status: $LAST_STATUS" >&2
    echo "Body:" >&2
    cat "$LAST_BODY_FILE" >&2
    echo >&2
    exit 1
  fi
  LOGIN_STATUS="$LAST_STATUS"
else
  LOGIN_STATUS=""
  LOGIN_URL=""
fi

REQUEST_URL="$(build_url "$PATH_ARG")"

BODY_MODE=""
BODY_VALUE=""
if [[ -n "$JSON_BODY" ]]; then
  BODY_MODE="json"
  BODY_VALUE="$JSON_BODY"
elif [[ -n "$JSON_FILE" ]]; then
  BODY_MODE="json_file"
  BODY_VALUE="$JSON_FILE"
elif [[ -n "$RAW_BODY" ]]; then
  BODY_MODE="raw"
  BODY_VALUE="$RAW_BODY"
elif [[ -n "$RAW_BODY_FILE" ]]; then
  BODY_MODE="raw_file"
  BODY_VALUE="$RAW_BODY_FILE"
fi

run_curl_request "$METHOD" "$REQUEST_URL" "$BODY_MODE" "$BODY_VALUE"
FINAL_STATUS="$LAST_STATUS"

echo "Repo root: $REPO_ROOT"
echo "Env file: $ENV_FILE"
echo "Base URL: $BASE_URL"
if [[ -n "$READY_STATUS" ]]; then
  echo "Ready check: $READY_STATUS $READY_URL"
fi
if [[ -n "$LOGIN_STATUS" ]]; then
  echo "Login: $LOGIN_STATUS $LOGIN_URL"
fi
METHOD_UPPER="$(printf '%s' "$METHOD" | tr '[:lower:]' '[:upper:]')"
echo "Request: $METHOD_UPPER $REQUEST_URL"
echo "Status: $FINAL_STATUS"
echo "Headers:"
cat "$LAST_HEADERS_FILE"
echo "Body:"
cat "$LAST_BODY_FILE"
echo

if [[ "$SHOW_COOKIES" -eq 1 && -n "$COOKIE_JAR" ]]; then
  echo "Cookies:"
  grep -v '^$' "$COOKIE_JAR" || true
fi

if [[ "$FAIL_STATUS" -eq 1 ]] && ! is_success_status "$FINAL_STATUS"; then
  exit 1
fi

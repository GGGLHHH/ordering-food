#!/usr/bin/env bash

set -euo pipefail

exec codex -c 'mcp_servers.dbhub.url="http://localhost:1000/mcp"' "$@"

---
name: request-ordering-food-api
description: Request, smoke-test, and debug any ordering-food REST API endpoint. Use when Codex needs to call project APIs, resolve the local server host and port from `.env`, confirm the service is running, authenticate with project credentials, or send requests with existing bearer tokens or access/refresh token cookies.
---

# Request Ordering Food API

Use the bundled shell script instead of ad hoc `curl` when working with this repository's REST API.

## Workflow

1. Resolve the base URL from `.env`.
   The script reads `APP__HOST` and `APP__PORT` from the repository root `.env`.
   Override them only when the user explicitly asks for a different host, port, or base URL.

2. Check the server before making the real request.
   By default the script calls `/health/ready`.
   If the service is not reachable or not ready, stop and either:
   - start the server with `make run` or `make dev`, or
   - tell the user the local API is not available yet.

3. Choose one authentication mode.
   - Public endpoint: send the request directly.
   - Account login: pass `--login-email` and `--login-password`, then let the script call `/api/auth/login` before the target request.
   - Existing tokens: pass `--bearer-token`, `--access-token`, `--refresh-token`, or custom `--cookie` values.

4. Send the target request with the script.
   Prefer `--json` or `--json-file` for JSON APIs.
   Use `--cookie-jar` when a multi-step flow needs to reuse cookies across calls.

## Script

Primary entrypoint:

```bash
bash .agents/skills/request-ordering-food-api/scripts/request_api.sh <path> [options]
```

Common examples:

```bash
bash .agents/skills/request-ordering-food-api/scripts/request_api.sh /health/ready
```

```bash
bash .agents/skills/request-ordering-food-api/scripts/request_api.sh /api/identity/users \
  --method POST \
  --json '{"display_name":"Alice","identities":[{"identity_type":"email","identifier":"alice@example.com"}],"password":"secret123"}'
```

```bash
bash .agents/skills/request-ordering-food-api/scripts/request_api.sh /api/auth/me \
  --login-email alice@example.com \
  --login-password secret123
```

```bash
bash .agents/skills/request-ordering-food-api/scripts/request_api.sh /api/auth/me \
  --access-token "$ACCESS_TOKEN"
```

```bash
bash .agents/skills/request-ordering-food-api/scripts/request_api.sh /api/auth/refresh \
  --refresh-token "$REFRESH_TOKEN"
```

```bash
bash .agents/skills/request-ordering-food-api/scripts/request_api.sh /api/auth/me \
  --login-email alice@example.com \
  --login-password secret123 \
  --cookie-jar /tmp/ordering-food-auth.cookies \
  --show-cookies
```

## Notes

- The implementation intentionally uses `bash` and `curl` only, so it does not depend on Python packages or `jq`.
- Keep `--skip-ready-check` for rare cases only, such as intentionally testing startup failures.
- Use `--fail-status` when you want the command to exit non-zero on 4xx or 5xx responses.
- Use `--show-cookies` only when the user explicitly needs token or cookie values in the output.
- For repeated protected requests in one session, prefer `--cookie-jar` over re-logging on every call.

import { getAuthMe, postAuthLogin, postAuthLogout } from '#/contracts/openapi/api'
import type { AuthMeResponse, AuthResponse, LoginRequest } from '#/contracts/openapi/types'
import { type AuthMode, requestJson, requestVoid } from '#/integrations/http'

interface AuthApiOptions {
  authMode?: AuthMode
  signal?: AbortSignal
}

export function login(payload: LoginRequest, signal?: AbortSignal) {
  return requestJson<AuthResponse>(postAuthLogin(), {
    authMode: 'none',
    json: payload,
    method: 'POST',
    signal,
  })
}

export function logout(signal?: AbortSignal) {
  return requestVoid(postAuthLogout(), {
    authMode: 'none',
    method: 'POST',
    signal,
  })
}

export function getCurrentUser(options: AuthApiOptions = {}) {
  return requestJson<AuthMeResponse>(getAuthMe(), {
    authMode: options.authMode ?? 'required',
    method: 'GET',
    signal: options.signal,
  })
}

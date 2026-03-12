import type { AuthMeResponse, AuthResponse, LoginRequest } from '#/contracts/generated'
import { type AuthMode, requestJson, requestVoid } from '#/integrations/http'

interface AuthApiOptions {
  authMode?: AuthMode
  signal?: AbortSignal
}

export function login(payload: LoginRequest, signal?: AbortSignal) {
  return requestJson<AuthResponse>('auth/login', {
    authMode: 'none',
    json: payload,
    method: 'POST',
    signal,
  })
}

export function logout(signal?: AbortSignal) {
  return requestVoid('auth/logout', {
    authMode: 'none',
    method: 'POST',
    signal,
  })
}

export function getCurrentUser(options: AuthApiOptions = {}) {
  return requestJson<AuthMeResponse>('auth/me', {
    authMode: options.authMode ?? 'required',
    method: 'GET',
    signal: options.signal,
  })
}

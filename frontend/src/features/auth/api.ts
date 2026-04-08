import { getAuthMe, postAuthLogin, postAuthLogout } from '#/contracts/openapi/client'
import type { LoginRequest } from '#/contracts/openapi/types'
import type { AuthMode } from '#/integrations/http'

interface AuthApiOptions {
  authMode?: AuthMode
  signal?: AbortSignal
}

export function login(payload: LoginRequest, signal?: AbortSignal) {
  return postAuthLogin(
    {
      body: payload,
      signal,
    },
    {
      authMode: 'none',
    },
  )
}

export function logout(signal?: AbortSignal) {
  return postAuthLogout(
    {
      signal,
    },
    {
      authMode: 'none',
    },
  )
}

export function getCurrentUser(options: AuthApiOptions = {}) {
  return getAuthMe(
    {
      signal: options.signal,
    },
    {
      authMode: options.authMode ?? 'required',
    },
  )
}

import type {
  BindIdentityUserIdentityRequest,
  CreateIdentityUserRequest,
  IdentityUserPath,
  IdentityUserResponse,
  UpdateIdentityUserProfileRequest,
} from '#/contracts/openapi/types'
import { requestJson } from '#/integrations/http'

export function createIdentityUser(payload: CreateIdentityUserRequest, signal?: AbortSignal) {
  return requestJson<IdentityUserResponse>('identity/users', {
    authMode: 'none',
    json: payload,
    method: 'POST',
    signal,
  })
}

export function getIdentityUser(path: IdentityUserPath, signal?: AbortSignal) {
  return requestJson<IdentityUserResponse>(`identity/users/${path.user_id}`, {
    authMode: 'none',
    method: 'GET',
    signal,
  })
}

export function updateIdentityUserProfile(
  path: IdentityUserPath,
  payload: UpdateIdentityUserProfileRequest,
  signal?: AbortSignal,
) {
  return requestJson<IdentityUserResponse>(`identity/users/${path.user_id}/profile`, {
    authMode: 'none',
    json: payload,
    method: 'PATCH',
    signal,
  })
}

export function bindIdentityUserIdentity(
  path: IdentityUserPath,
  payload: BindIdentityUserIdentityRequest,
  signal?: AbortSignal,
) {
  return requestJson<IdentityUserResponse>(`identity/users/${path.user_id}/identities`, {
    authMode: 'none',
    json: payload,
    method: 'POST',
    signal,
  })
}

export function disableIdentityUser(path: IdentityUserPath, signal?: AbortSignal) {
  return requestJson<IdentityUserResponse>(`identity/users/${path.user_id}/disable`, {
    authMode: 'none',
    method: 'POST',
    signal,
  })
}

export function softDeleteIdentityUser(path: IdentityUserPath, signal?: AbortSignal) {
  return requestJson<IdentityUserResponse>(`identity/users/${path.user_id}/soft-delete`, {
    authMode: 'none',
    method: 'POST',
    signal,
  })
}

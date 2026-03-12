import type {
  BindIdentityUserIdentityRequest,
  CreateIdentityUserRequest,
  IdentityUserResponse,
  UpdateIdentityUserProfileRequest,
} from '#/contracts/generated'
import { requestJson } from '#/integrations/http'

export function createIdentityUser(payload: CreateIdentityUserRequest, signal?: AbortSignal) {
  return requestJson<IdentityUserResponse>('identity/users', {
    authMode: 'none',
    json: payload,
    method: 'POST',
    signal,
  })
}

export function getIdentityUser(userId: string, signal?: AbortSignal) {
  return requestJson<IdentityUserResponse>(`identity/users/${userId}`, {
    authMode: 'none',
    method: 'GET',
    signal,
  })
}

export function updateIdentityUserProfile(
  userId: string,
  payload: UpdateIdentityUserProfileRequest,
  signal?: AbortSignal,
) {
  return requestJson<IdentityUserResponse>(`identity/users/${userId}/profile`, {
    authMode: 'none',
    json: payload,
    method: 'PATCH',
    signal,
  })
}

export function bindIdentityUserIdentity(
  userId: string,
  payload: BindIdentityUserIdentityRequest,
  signal?: AbortSignal,
) {
  return requestJson<IdentityUserResponse>(`identity/users/${userId}/identities`, {
    authMode: 'none',
    json: payload,
    method: 'POST',
    signal,
  })
}

export function disableIdentityUser(userId: string, signal?: AbortSignal) {
  return requestJson<IdentityUserResponse>(`identity/users/${userId}/disable`, {
    authMode: 'none',
    method: 'POST',
    signal,
  })
}

export function softDeleteIdentityUser(userId: string, signal?: AbortSignal) {
  return requestJson<IdentityUserResponse>(`identity/users/${userId}/soft-delete`, {
    authMode: 'none',
    method: 'POST',
    signal,
  })
}

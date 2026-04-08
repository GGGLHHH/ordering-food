import {
  getIdentityUser,
  patchIdentityUserProfile,
  postIdentityUserDisable,
  postIdentityUserIdentities,
  postIdentityUserSoftDelete,
  postIdentityUsers,
} from '#/contracts/openapi/api'
import type {
  BindIdentityUserIdentityRequest,
  CreateIdentityUserRequest,
  IdentityUserPath,
  IdentityUserResponse,
  UpdateIdentityUserProfileRequest,
} from '#/contracts/openapi/types'
import { requestJson } from '#/integrations/http'

export function createIdentityUser(payload: CreateIdentityUserRequest, signal?: AbortSignal) {
  return requestJson<IdentityUserResponse>(postIdentityUsers(), {
    authMode: 'none',
    json: payload,
    method: 'POST',
    signal,
  })
}

export function fetchIdentityUser(path: IdentityUserPath, signal?: AbortSignal) {
  return requestJson<IdentityUserResponse>(getIdentityUser({ user_id: path.user_id }), {
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
  return requestJson<IdentityUserResponse>(patchIdentityUserProfile({ user_id: path.user_id }), {
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
  return requestJson<IdentityUserResponse>(postIdentityUserIdentities({ user_id: path.user_id }), {
    authMode: 'none',
    json: payload,
    method: 'POST',
    signal,
  })
}

export function disableIdentityUser(path: IdentityUserPath, signal?: AbortSignal) {
  return requestJson<IdentityUserResponse>(postIdentityUserDisable({ user_id: path.user_id }), {
    authMode: 'none',
    method: 'POST',
    signal,
  })
}

export function softDeleteIdentityUser(path: IdentityUserPath, signal?: AbortSignal) {
  return requestJson<IdentityUserResponse>(postIdentityUserSoftDelete({ user_id: path.user_id }), {
    authMode: 'none',
    method: 'POST',
    signal,
  })
}

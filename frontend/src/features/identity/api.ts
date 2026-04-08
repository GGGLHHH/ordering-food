import {
  getIdentityUser,
  patchIdentityUserProfile,
  postIdentityUserDisable,
  postIdentityUserIdentities,
  postIdentityUserSoftDelete,
  postIdentityUsers,
} from '#/contracts/openapi/client'
import type {
  BindIdentityUserIdentityRequest,
  CreateIdentityUserRequest,
  IdentityUserPath,
  UpdateIdentityUserProfileRequest,
} from '#/contracts/openapi/types'

export function createIdentityUser(payload: CreateIdentityUserRequest, signal?: AbortSignal) {
  return postIdentityUsers(
    {
      body: payload,
      signal,
    },
    {
      authMode: 'none',
    },
  )
}

export function fetchIdentityUser(path: IdentityUserPath, signal?: AbortSignal) {
  return getIdentityUser(
    {
      path,
      signal,
    },
    {
      authMode: 'none',
    },
  )
}

export function updateIdentityUserProfile(
  path: IdentityUserPath,
  payload: UpdateIdentityUserProfileRequest,
  signal?: AbortSignal,
) {
  return patchIdentityUserProfile(
    {
      body: payload,
      path,
      signal,
    },
    {
      authMode: 'none',
    },
  )
}

export function bindIdentityUserIdentity(
  path: IdentityUserPath,
  payload: BindIdentityUserIdentityRequest,
  signal?: AbortSignal,
) {
  return postIdentityUserIdentities(
    {
      body: payload,
      path,
      signal,
    },
    {
      authMode: 'none',
    },
  )
}

export function disableIdentityUser(path: IdentityUserPath, signal?: AbortSignal) {
  return postIdentityUserDisable(
    {
      path,
      signal,
    },
    {
      authMode: 'none',
    },
  )
}

export function softDeleteIdentityUser(path: IdentityUserPath, signal?: AbortSignal) {
  return postIdentityUserSoftDelete(
    {
      path,
      signal,
    },
    {
      authMode: 'none',
    },
  )
}

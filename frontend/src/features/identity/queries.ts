import { mutationOptions, queryOptions, useMutation } from '@tanstack/react-query'

import type {
  BindIdentityUserIdentityRequest,
  CreateIdentityUserRequest,
  IdentityUserPath,
  UpdateIdentityUserProfileRequest,
} from '#/contracts/openapi/types'

import {
  bindIdentityUserIdentity,
  createIdentityUser,
  disableIdentityUser,
  getIdentityUser,
  softDeleteIdentityUser,
  updateIdentityUserProfile,
} from './api'

export const identityKeys = {
  all: ['identity'] as const,
  detail: (userId: string) => [...identityKeys.all, 'detail', userId] as const,
}

export const identityQueries = {
  detail: (path: IdentityUserPath) =>
    queryOptions({
      queryFn: ({ signal }) => getIdentityUser(path, signal),
      queryKey: identityKeys.detail(path.user_id),
    }),
}

export const identityMutations = {
  bindIdentity: (path: IdentityUserPath) =>
    mutationOptions({
      mutationFn: (payload: BindIdentityUserIdentityRequest) =>
        bindIdentityUserIdentity(path, payload),
      mutationKey: [...identityKeys.all, 'bindIdentity', path.user_id] as const,
    }),
  create: () =>
    mutationOptions({
      mutationFn: (payload: CreateIdentityUserRequest) => createIdentityUser(payload),
      mutationKey: [...identityKeys.all, 'create'] as const,
    }),
  disable: (path: IdentityUserPath) =>
    mutationOptions({
      mutationFn: () => disableIdentityUser(path),
      mutationKey: [...identityKeys.all, 'disable', path.user_id] as const,
    }),
  softDelete: (path: IdentityUserPath) =>
    mutationOptions({
      mutationFn: () => softDeleteIdentityUser(path),
      mutationKey: [...identityKeys.all, 'softDelete', path.user_id] as const,
    }),
  updateProfile: (path: IdentityUserPath) =>
    mutationOptions({
      mutationFn: (payload: UpdateIdentityUserProfileRequest) =>
        updateIdentityUserProfile(path, payload),
      mutationKey: [...identityKeys.all, 'updateProfile', path.user_id] as const,
    }),
}

export function useCreateIdentityUserMutation() {
  return useMutation({
    ...identityMutations.create(),
  })
}

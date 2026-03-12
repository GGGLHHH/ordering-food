import { mutationOptions, queryOptions, useMutation } from '@tanstack/react-query'
import type {
  BindIdentityUserIdentityRequest,
  CreateIdentityUserRequest,
  UpdateIdentityUserProfileRequest,
} from '#/contracts/generated'
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
  detail: (userId: string) =>
    queryOptions({
      queryFn: ({ signal }) => getIdentityUser(userId, signal),
      queryKey: identityKeys.detail(userId),
    }),
}

export const identityMutations = {
  bindIdentity: (userId: string) =>
    mutationOptions({
      mutationFn: (payload: BindIdentityUserIdentityRequest) =>
        bindIdentityUserIdentity(userId, payload),
      mutationKey: [...identityKeys.all, 'bindIdentity', userId] as const,
    }),
  create: () =>
    mutationOptions({
      mutationFn: (payload: CreateIdentityUserRequest) => createIdentityUser(payload),
      mutationKey: [...identityKeys.all, 'create'] as const,
    }),
  disable: (userId: string) =>
    mutationOptions({
      mutationFn: () => disableIdentityUser(userId),
      mutationKey: [...identityKeys.all, 'disable', userId] as const,
    }),
  softDelete: (userId: string) =>
    mutationOptions({
      mutationFn: () => softDeleteIdentityUser(userId),
      mutationKey: [...identityKeys.all, 'softDelete', userId] as const,
    }),
  updateProfile: (userId: string) =>
    mutationOptions({
      mutationFn: (payload: UpdateIdentityUserProfileRequest) =>
        updateIdentityUserProfile(userId, payload),
      mutationKey: [...identityKeys.all, 'updateProfile', userId] as const,
    }),
}

export function useCreateIdentityUserMutation() {
  return useMutation({
    ...identityMutations.create(),
  })
}

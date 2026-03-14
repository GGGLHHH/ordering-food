import type { QueryClient } from '@tanstack/react-query'
import {
  mutationOptions,
  queryOptions,
  useMutation,
  useQuery,
  useQueryClient,
} from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'

import type { AuthMeResponse, LoginRequest } from '#/contracts/generated'
import { ApiError, isAbortError } from '#/integrations/http'

import { getCurrentUser, login, logout } from './api'

export const authKeys = {
  all: ['auth'] as const,
  me: () => [...authKeys.all, 'me'] as const,
}

export const authQueries = {
  me: () =>
    queryOptions({
      queryFn: async ({ signal }) => {
        try {
          return await getCurrentUser({
            authMode: 'optional',
            signal,
          })
        } catch (error) {
          if (isAbortError(error)) {
            throw error
          }

          if (error instanceof ApiError && error.isUnauthorized) {
            return null
          }

          throw error
        }
      },
      queryKey: authKeys.me(),
      staleTime: 60_000,
    }),
}

export const authMutations = {
  login: () =>
    mutationOptions({
      mutationFn: (payload: LoginRequest) => login(payload),
      mutationKey: [...authKeys.all, 'login'] as const,
    }),
  logout: () =>
    mutationOptions({
      mutationFn: () => logout(),
      mutationKey: [...authKeys.all, 'logout'] as const,
    }),
}

export function useAuthSessionQuery() {
  return useQuery({
    ...authQueries.me(),
    enabled: typeof window !== 'undefined',
    placeholderData: null as AuthMeResponse | null,
  })
}

export function useLoginMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    ...authMutations.login(),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: authKeys.me(),
      })
    },
  })
}

export function useLogoutMutation() {
  const navigate = useNavigate()
  const queryClient = useQueryClient()

  return useMutation({
    ...authMutations.logout(),
    onSuccess: async () => {
      queryClient.removeQueries({
        queryKey: authKeys.all,
      })
      await navigate({
        to: '/login',
      })
    },
  })
}

export function refetchCurrentUser(queryClient: QueryClient) {
  return queryClient.fetchQuery(authQueries.me())
}

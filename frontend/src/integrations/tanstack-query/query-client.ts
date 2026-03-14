import { QueryClient } from '@tanstack/react-query'
import { ApiError } from '#/integrations/http'

const MAX_QUERY_RETRIES = 3

let browserQueryClient: QueryClient | undefined

export function createAppQueryClient() {
  return new QueryClient({
    defaultOptions: {
      mutations: {
        retry: 0,
      },
      queries: {
        retry: shouldRetryQuery,
      },
    },
  })
}

export function getQueryClient() {
  browserQueryClient ??= createAppQueryClient()
  return browserQueryClient
}

export function shouldRetryQuery(failureCount: number, error: unknown) {
  if (failureCount >= MAX_QUERY_RETRIES) {
    return false
  }

  return error instanceof ApiError ? error.isRetryable : false
}

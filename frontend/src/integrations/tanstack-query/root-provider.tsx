import { QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import { getQueryClient } from './query-client'

export function getContext() {
  return {
    queryClient: getQueryClient(),
  }
}

export default function TanStackQueryProvider({ children }: { children: ReactNode }) {
  const queryClient = getQueryClient()

  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
}

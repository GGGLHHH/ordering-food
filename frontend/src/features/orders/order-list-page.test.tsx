// @vitest-environment jsdom

import { QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vite-plus/test'

import { createAppQueryClient } from '#/integrations/tanstack-query/query-client'

import { OrderListPage } from './order-list-page'

vi.mock('@tanstack/react-router', () => ({
  Link: ({ children, ...props }: React.ComponentProps<'a'>) => <a {...props}>{children}</a>,
}))

describe('order list page', () => {
  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('renders customer orders from the API', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn<typeof fetch>(async (input) => {
        const url = getRequestUrl(input)
        if (url.endsWith('/api/orders')) {
          return createJsonResponse({
            orders: [
              {
                created_at: '2026-03-15T02:00:00Z',
                item_count: 2,
                order_id: 'order-1',
                status: 'accepted',
                store_id: 'store-1',
                subtotal_amount: 2100,
                total_amount: 2100,
                updated_at: '2026-03-15T02:05:00Z',
              },
            ],
          })
        }

        return createJsonResponse({}, 404)
      }),
    )
    const queryClient = createAppQueryClient()

    render(
      <QueryClientProvider client={queryClient}>
        <OrderListPage />
      </QueryClientProvider>,
    )

    await waitFor(() => {
      expect(screen.getByText('Accepted')).not.toBeNull()
    })

    expect(screen.getByText(/2 items • updated/i)).not.toBeNull()
    expect(screen.getByText(/¥21\.00|￥21\.00/)).not.toBeNull()
  })
})

function createJsonResponse(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    headers: {
      'content-type': 'application/json',
    },
    status,
  })
}

function getRequestUrl(input: RequestInfo | URL | undefined) {
  if (!input) {
    return ''
  }

  if (input instanceof Request) {
    return input.url
  }

  return String(input)
}

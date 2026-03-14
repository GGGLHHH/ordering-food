// @vitest-environment jsdom

import { QueryClientProvider } from '@tanstack/react-query'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vite-plus/test'

import { createAppQueryClient } from '#/integrations/tanstack-query/query-client'

import { readRecentOrderId, useOrderQuery, useOrdersQuery, usePlaceOrderMutation } from './queries'

describe('order query integration', () => {
  afterEach(() => {
    vi.restoreAllMocks()
    window.localStorage.clear()
  })

  it('fetches order detail and caches placed orders', async () => {
    const fetchMock = vi.fn<typeof fetch>(async (input) => {
      const url = getRequestUrl(input)
      if (url.endsWith('/api/orders/order-1')) {
        return createJsonResponse({
          created_at: '2026-03-15T01:00:00Z',
          customer_id: 'user-1',
          items: [],
          order_id: 'order-1',
          status: 'accepted',
          store_id: 'store-1',
          subtotal_amount: 1200,
          total_amount: 1200,
          updated_at: '2026-03-15T01:05:00Z',
        })
      }

      if (url.endsWith('/api/orders')) {
        if (input instanceof Request && input.method === 'GET') {
          return createJsonResponse({
            orders: [
              {
                created_at: '2026-03-15T01:00:00Z',
                item_count: 1,
                order_id: 'order-1',
                status: 'accepted',
                store_id: 'store-1',
                subtotal_amount: 1200,
                total_amount: 1200,
                updated_at: '2026-03-15T01:05:00Z',
              },
            ],
          })
        }

        return createJsonResponse({
          created_at: '2026-03-15T01:00:00Z',
          customer_id: 'user-1',
          items: [],
          order_id: 'order-2',
          status: 'pending_acceptance',
          store_id: 'store-1',
          subtotal_amount: 1200,
          total_amount: 1200,
          updated_at: '2026-03-15T01:00:00Z',
        })
      }

      return createJsonResponse({}, 404)
    })
    vi.stubGlobal('fetch', fetchMock)
    const queryClient = createAppQueryClient()

    render(
      <QueryClientProvider client={queryClient}>
        <>
          <OrderQueryProbe />
          <OrdersQueryProbe />
          <PlaceOrderProbe />
        </>
      </QueryClientProvider>,
    )

    await waitFor(() => {
      expect(screen.getByText('accepted')).not.toBeNull()
    })
    await waitFor(() => {
      expect(screen.getByText('list:1')).not.toBeNull()
    })

    fireEvent.click(screen.getByRole('button', { name: 'place-order' }))

    await waitFor(() => {
      expect(screen.getByText('placed:order-2')).not.toBeNull()
    })
  })
})

function OrderQueryProbe() {
  const query = useOrderQuery('order-1')
  return <p>{query.data?.status ?? 'loading'}</p>
}

function PlaceOrderProbe() {
  const mutation = usePlaceOrderMutation()

  return (
    <>
      <button
        type="button"
        onClick={() => {
          void mutation.mutateAsync({
            items: [],
            store_id: 'store-1',
          })
        }}
      >
        place-order
      </button>
      <p>{mutation.data ? `placed:${mutation.data.order_id}` : readRecentOrderId() ?? 'none'}</p>
    </>
  )
}

function OrdersQueryProbe() {
  const query = useOrdersQuery()
  return <p>{query.data ? `list:${query.data.orders.length}` : 'list:loading'}</p>
}

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

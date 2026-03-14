// @vitest-environment jsdom

import { QueryClientProvider } from '@tanstack/react-query'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vite-plus/test'

import { createAppQueryClient } from '#/integrations/tanstack-query/query-client'

import { OrderPage } from './order-page'

vi.mock('@tanstack/react-router', () => ({
  Link: ({ children, ...props }: React.ComponentProps<'a'>) => <a {...props}>{children}</a>,
}))

describe('order page', () => {
  afterEach(() => {
    vi.restoreAllMocks()
    window.localStorage.clear()
  })

  it('renders order detail and refreshes status', async () => {
    const fetchMock = vi.fn<typeof fetch>(async (input) => {
      const url = getRequestUrl(input)
      if (url.endsWith('/api/orders/order-1')) {
        return createJsonResponse({
          created_at: '2026-03-15T01:00:00Z',
          customer_id: 'user-1',
          items: [
            {
              line_number: 1,
              line_total_amount: 1200,
              menu_item_id: 'item-1',
              name: 'Garlic Cucumber Salad',
              quantity: 1,
              unit_price_amount: 1200,
            },
          ],
          order_id: 'order-1',
          status: 'ready_for_pickup',
          store_id: 'store-1',
          subtotal_amount: 1200,
          total_amount: 1200,
          updated_at: '2026-03-15T01:05:00Z',
        })
      }

      return createJsonResponse({}, 404)
    })
    vi.stubGlobal('fetch', fetchMock)
    const queryClient = createAppQueryClient()

    render(
      <QueryClientProvider client={queryClient}>
        <OrderPage orderId="order-1" />
      </QueryClientProvider>,
    )

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Ready For Pickup' })).not.toBeNull()
    })

    fireEvent.click(screen.getByRole('button', { name: 'Refresh status' }))

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(2)
    })
  })

  it('shows an unavailable state for API errors', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn<typeof fetch>(async () =>
        createJsonResponse({ code: 'not_found', message: 'order was not found' }, 404),
      ),
    )
    const queryClient = createAppQueryClient()

    render(
      <QueryClientProvider client={queryClient}>
        <OrderPage orderId="order-missing" />
      </QueryClientProvider>,
    )

    await waitFor(() => {
      expect(screen.getByText('Order unavailable')).not.toBeNull()
    })
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

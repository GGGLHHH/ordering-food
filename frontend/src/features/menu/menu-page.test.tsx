// @vitest-environment jsdom

import { QueryClientProvider } from '@tanstack/react-query'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vite-plus/test'
import { createAppQueryClient } from '#/integrations/tanstack-query/query-client'
import { MenuPage } from './menu-page'

describe('menu page', () => {
  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('renders menu data and notifies category changes', async () => {
    const onCategoryChange = vi.fn()
    const fetchMock = vi.fn<typeof fetch>(async (input) => {
      const url = getRequestUrl(input)
      if (url.endsWith('/api/menu/store')) {
        return createJsonResponse({
          currency_code: 'CNY',
          name: 'Ordering Food Demo Kitchen',
          slug: 'ordering-food-demo',
          status: 'active',
          store_id: 'store-1',
          timezone: 'Asia/Shanghai',
        })
      }

      if (url.endsWith('/api/menu/categories')) {
        return createJsonResponse({
          categories: [
            {
              category_id: 'category-featured',
              description: 'Popular picks for first-time visitors.',
              name: 'Featured',
              slug: 'featured',
              sort_order: 10,
              status: 'active',
              store_id: 'store-1',
            },
            {
              category_id: 'category-mains',
              description: 'Comfort food staples and filling bowls.',
              name: 'Mains',
              slug: 'mains',
              sort_order: 20,
              status: 'active',
              store_id: 'store-1',
            },
          ],
        })
      }

      if (url.includes('/api/menu/items')) {
        return createJsonResponse({
          items: [
            {
              category_id: 'category-featured',
              description: 'Golden chicken, soft egg, pickled greens, and rice.',
              item_id: 'item-1',
              name: 'Crispy Chicken Bowl',
              price_amount: 3200,
              slug: 'crispy-chicken-bowl',
              sort_order: 10,
              status: 'active',
              store_id: 'store-1',
            },
          ],
        })
      }

      return createJsonResponse({}, 404)
    })
    vi.stubGlobal('fetch', fetchMock)
    const queryClient = createAppQueryClient()

    render(
      <QueryClientProvider client={queryClient}>
        <MenuPage onCategoryChange={onCategoryChange} selectedCategorySlug="featured" />
      </QueryClientProvider>,
    )

    await waitFor(() => {
      expect(screen.getByText('Ordering Food Demo Kitchen')).not.toBeNull()
    })

    expect(screen.getByText('Crispy Chicken Bowl')).not.toBeNull()
    expect(screen.getByText(/¥32\.00|￥32\.00/)).not.toBeNull()

    fireEvent.click(screen.getByRole('button', { name: /Mains/i }))

    expect(onCategoryChange).toHaveBeenCalledWith('mains')
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

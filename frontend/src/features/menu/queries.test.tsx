// @vitest-environment jsdom

import { QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vite-plus/test'
import { createAppQueryClient } from '#/integrations/tanstack-query/query-client'
import { useMenuItemsQuery } from './queries'

describe('menu query integration', () => {
  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('requests menu items with category slug filters', async () => {
    const fetchMock = vi.fn<typeof fetch>(async (input) => {
      const url = getRequestUrl(input)
      if (url.includes('/api/menu/items')) {
        return createJsonResponse({
          items: [
            {
              category_id: 'category-featured',
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
        <MenuItemsProbe categorySlug="featured" />
      </QueryClientProvider>,
    )

    await waitFor(() => {
      expect(screen.getByText('1')).not.toBeNull()
    })

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(getRequestUrl(fetchMock.mock.calls[0]?.[0])).toContain(
      '/api/menu/items?category_slug=featured',
    )
  })
})

function MenuItemsProbe({ categorySlug }: { categorySlug?: string }) {
  const query = useMenuItemsQuery(categorySlug)
  return <p>{query.data?.items.length ?? 0}</p>
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

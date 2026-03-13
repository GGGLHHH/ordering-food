import type {
  MenuCategoriesResponse,
  MenuItemResponse,
  MenuItemsQuery,
  MenuItemsResponse,
  MenuStoreResponse,
} from '#/contracts/generated'
import { requestJson } from '#/integrations/http'

export function getMenuStore(signal?: AbortSignal) {
  return requestJson<MenuStoreResponse>('menu/store', {
    authMode: 'none',
    method: 'GET',
    signal,
  })
}

export function getMenuCategories(signal?: AbortSignal) {
  return requestJson<MenuCategoriesResponse>('menu/categories', {
    authMode: 'none',
    method: 'GET',
    signal,
  })
}

export function getMenuItems(query: MenuItemsQuery = {}, signal?: AbortSignal) {
  return requestJson<MenuItemsResponse>('menu/items', {
    authMode: 'none',
    method: 'GET',
    searchParams: sanitizeMenuItemsQuery(query),
    signal,
  })
}

export function getMenuItem(itemId: string, signal?: AbortSignal) {
  return requestJson<MenuItemResponse>(`menu/items/${itemId}`, {
    authMode: 'none',
    method: 'GET',
    signal,
  })
}

function sanitizeMenuItemsQuery(query: MenuItemsQuery) {
  const searchParams = new URLSearchParams()

  if (query.category_id?.trim()) {
    searchParams.set('category_id', query.category_id.trim())
  }

  if (query.category_slug?.trim()) {
    searchParams.set('category_slug', query.category_slug.trim())
  }

  return searchParams
}

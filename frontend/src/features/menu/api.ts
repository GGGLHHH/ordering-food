import {
  CATALOG_CATEGORIES,
  CATALOG_ITEMS,
  CATALOG_STORE,
  catalogItemPath,
} from '#/contracts/openapi/helpers'
import type {
  MenuCategoriesResponse,
  MenuItemResponse,
  MenuItemsQuery,
  MenuItemsResponse,
  MenuStoreResponse,
} from '#/contracts/openapi/types'
import { requestJson } from '#/integrations/http'

export function getMenuStore(signal?: AbortSignal) {
  return requestJson<MenuStoreResponse>(CATALOG_STORE, {
    authMode: 'none',
    method: 'GET',
    signal,
  })
}

export function getMenuCategories(signal?: AbortSignal) {
  return requestJson<MenuCategoriesResponse>(CATALOG_CATEGORIES, {
    authMode: 'none',
    method: 'GET',
    signal,
  })
}

export function getMenuItems(query: MenuItemsQuery = {}, signal?: AbortSignal) {
  return requestJson<MenuItemsResponse>(CATALOG_ITEMS, {
    authMode: 'none',
    method: 'GET',
    searchParams: sanitizeMenuItemsQuery(query),
    signal,
  })
}

export function getMenuItem(itemId: string, signal?: AbortSignal) {
  return requestJson<MenuItemResponse>(catalogItemPath(itemId), {
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

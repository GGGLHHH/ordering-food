import {
  getCatalogCategories,
  getCatalogItem,
  getCatalogItems,
  getCatalogStore,
} from '#/contracts/openapi/api'
import type {
  MenuCategoriesResponse,
  MenuItemPath,
  MenuItemResponse,
  MenuItemsQuery,
  MenuItemsResponse,
  MenuStoreResponse,
} from '#/contracts/openapi/types'
import { requestJson } from '#/integrations/http'

export function getMenuStore(signal?: AbortSignal) {
  return requestJson<MenuStoreResponse>(getCatalogStore(), {
    authMode: 'none',
    method: 'GET',
    signal,
  })
}

export function getMenuCategories(signal?: AbortSignal) {
  return requestJson<MenuCategoriesResponse>(getCatalogCategories(), {
    authMode: 'none',
    method: 'GET',
    signal,
  })
}

export function getMenuItems(query: MenuItemsQuery = {}, signal?: AbortSignal) {
  return requestJson<MenuItemsResponse>(
    getCatalogItems({
      category_id: query.category_id ?? null,
      category_slug: query.category_slug ?? null,
    }),
    {
      authMode: 'none',
      method: 'GET',
      signal,
    },
  )
}

export function getMenuItem(path: MenuItemPath, signal?: AbortSignal) {
  return requestJson<MenuItemResponse>(getCatalogItem({ item_id: path.item_id }), {
    authMode: 'none',
    method: 'GET',
    signal,
  })
}

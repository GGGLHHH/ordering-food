import {
  getCatalogCategories,
  getCatalogItem,
  getCatalogItems,
  getCatalogStore,
} from '#/contracts/openapi/client'
import type { MenuItemPath, MenuItemsQuery } from '#/contracts/openapi/types'

export function getMenuStore(signal?: AbortSignal) {
  return getCatalogStore(
    {
      signal,
    },
    {
      authMode: 'none',
    },
  )
}

export function getMenuCategories(signal?: AbortSignal) {
  return getCatalogCategories(
    {
      signal,
    },
    {
      authMode: 'none',
    },
  )
}

export function getMenuItems(query: MenuItemsQuery = {}, signal?: AbortSignal) {
  return getCatalogItems(
    {
      query,
      signal,
    },
    {
      authMode: 'none',
    },
  )
}

export function getMenuItem(path: MenuItemPath, signal?: AbortSignal) {
  return getCatalogItem(
    {
      path,
      signal,
    },
    {
      authMode: 'none',
    },
  )
}

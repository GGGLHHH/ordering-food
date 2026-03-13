import { queryOptions, useQuery } from '@tanstack/react-query'
import { getMenuCategories, getMenuItem, getMenuItems, getMenuStore } from './api'

export const menuKeys = {
  all: ['menu'] as const,
  categories: () => [...menuKeys.all, 'categories'] as const,
  items: (categorySlug?: string) => [...menuKeys.all, 'items', categorySlug ?? 'all'] as const,
  item: (itemId: string) => [...menuKeys.all, 'item', itemId] as const,
  store: () => [...menuKeys.all, 'store'] as const,
}

export const menuQueries = {
  store: () =>
    queryOptions({
      queryFn: ({ signal }) => getMenuStore(signal),
      queryKey: menuKeys.store(),
      staleTime: 60_000,
    }),
  categories: () =>
    queryOptions({
      queryFn: ({ signal }) => getMenuCategories(signal),
      queryKey: menuKeys.categories(),
      staleTime: 60_000,
    }),
  items: (categorySlug?: string) =>
    queryOptions({
      queryFn: ({ signal }) =>
        getMenuItems(
          categorySlug
            ? {
                category_slug: categorySlug,
              }
            : {},
          signal,
        ),
      queryKey: menuKeys.items(categorySlug),
      staleTime: 30_000,
    }),
  item: (itemId: string) =>
    queryOptions({
      queryFn: ({ signal }) => getMenuItem(itemId, signal),
      queryKey: menuKeys.item(itemId),
      staleTime: 30_000,
    }),
}

export function useMenuStoreQuery() {
  return useQuery(menuQueries.store())
}

export function useMenuCategoriesQuery() {
  return useQuery(menuQueries.categories())
}

export function useMenuItemsQuery(categorySlug?: string) {
  return useQuery(menuQueries.items(categorySlug))
}

export function useMenuItemQuery(itemId: string) {
  return useQuery({
    ...menuQueries.item(itemId),
    enabled: itemId.trim().length > 0,
  })
}

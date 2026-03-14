import { mutationOptions, queryOptions } from '@tanstack/react-query'

import type { ExampleItemPath, ExamplePayload, ExampleSearchQuery } from '#/contracts/generated'

import { echoExamplePayload, getExampleItem, searchExamples } from './api'

export const exampleKeys = {
  all: ['examples'] as const,
  item: (itemId: number) => [...exampleKeys.all, 'item', itemId] as const,
  search: (page: number) => [...exampleKeys.all, 'search', { page }] as const,
}

export const exampleQueries = {
  item: (path: ExampleItemPath) =>
    queryOptions({
      queryFn: ({ signal }) => getExampleItem(path, signal),
      queryKey: exampleKeys.item(path.item_id),
    }),
  search: (query: ExampleSearchQuery) =>
    queryOptions({
      queryFn: ({ signal }) => searchExamples(query, signal),
      queryKey: exampleKeys.search(query.page),
    }),
}

export const exampleMutations = {
  echo: () =>
    mutationOptions({
      mutationFn: (payload: ExamplePayload) => echoExamplePayload(payload),
      mutationKey: [...exampleKeys.all, 'echo'] as const,
    }),
}

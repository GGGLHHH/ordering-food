import { getExamplesItem, getExamplesSearch, postExamplesEcho } from '#/contracts/openapi/client'
import type { ExampleItemPath, ExamplePayload, ExampleSearchQuery } from '#/contracts/openapi/types'

export function echoExamplePayload(payload: ExamplePayload, signal?: AbortSignal) {
  return postExamplesEcho(
    {
      body: payload,
      signal,
    },
    {
      authMode: 'none',
    },
  )
}

export function searchExamples(query: ExampleSearchQuery, signal?: AbortSignal) {
  return getExamplesSearch(
    {
      query,
      signal,
    },
    {
      authMode: 'none',
    },
  )
}

export function getExampleItem(path: ExampleItemPath, signal?: AbortSignal) {
  return getExamplesItem(
    {
      path,
      signal,
    },
    {
      authMode: 'none',
    },
  )
}

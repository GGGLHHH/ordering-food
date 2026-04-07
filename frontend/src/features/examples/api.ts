import type {
  ExampleItemPath,
  ExampleItemResponse,
  ExamplePayload,
  ExamplePayloadResponse,
  ExampleSearchQuery,
  ExampleSearchResponse,
} from '#/contracts/openapi/types'
import { requestJson } from '#/integrations/http'

export function echoExamplePayload(payload: ExamplePayload, signal?: AbortSignal) {
  return requestJson<ExamplePayloadResponse>('examples/echo', {
    authMode: 'none',
    json: payload,
    method: 'POST',
    signal,
  })
}

export function searchExamples(query: ExampleSearchQuery, signal?: AbortSignal) {
  return requestJson<ExampleSearchResponse>('examples/search', {
    authMode: 'none',
    method: 'GET',
    searchParams: { page: String(query.page) },
    signal,
  })
}

export function getExampleItem(path: ExampleItemPath, signal?: AbortSignal) {
  return requestJson<ExampleItemResponse>(`examples/items/${path.item_id}`, {
    authMode: 'none',
    method: 'GET',
    signal,
  })
}

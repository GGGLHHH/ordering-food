import { getExamplesItem, getExamplesSearch, postExamplesEcho } from '#/contracts/openapi/api'
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
  return requestJson<ExamplePayloadResponse>(postExamplesEcho(), {
    authMode: 'none',
    json: payload,
    method: 'POST',
    signal,
  })
}

export function searchExamples(query: ExampleSearchQuery, signal?: AbortSignal) {
  return requestJson<ExampleSearchResponse>(getExamplesSearch({ page: query.page }), {
    authMode: 'none',
    method: 'GET',
    signal,
  })
}

export function getExampleItem(path: ExampleItemPath, signal?: AbortSignal) {
  return requestJson<ExampleItemResponse>(getExamplesItem({ item_id: path.item_id }), {
    authMode: 'none',
    method: 'GET',
    signal,
  })
}

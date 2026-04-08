import { describe, expect, it } from 'vite-plus/test'

import { renderGeneratedArtifacts } from './vite-plugin-openapi-codegen'

describe('vite-plugin-openapi-codegen', () => {
  it('generates path builders and typed request client functions', () => {
    const files = renderGeneratedArtifacts(createSpec(), {})

    expect(files.api).toContain('export function postAuthLogin(): string {')
    expect(files.api).toContain('export function getCatalogItem(')
    expect(files.api).toContain('export function getExamplesSearch(): string {')
    expect(files.api).not.toContain('buildQuery')

    expect(files.client).toContain("import type { ApiRequestOptions } from '#/integrations/http'")
    expect(files.client).toContain("import { requestJson, requestVoid } from '#/integrations/http'")
    expect(files.client).toContain('import type {')
    expect(files.client).toContain('  AuthResponse,')
    expect(files.client).toContain('  CatalogItemResponse,')
    expect(files.client).toContain('  ExampleSearchResponse,')
    expect(files.client).toContain('  LoginRequest,')
    expect(files.client).toContain('  ReviewRequest,')
    expect(files.client).toContain('  ReviewResponse,')
    expect(files.client).toContain("} from './types'")
    expect(files.client).toContain('import {')
    expect(files.client).toContain("from './api'")

    expect(files.client).toContain('export interface PostAuthLoginOptions {')
    expect(files.client).toContain('query?: never')
    expect(files.client).toContain('path?: never')
    expect(files.client).toContain('body: LoginRequest')
    expect(files.client).toContain('signal?: AbortSignal')
    expect(files.client).toContain('): Promise<AuthResponse> {')
    expect(files.client).not.toContain(
      "operations['login']['requestBody']['content']['application/json']",
    )
    expect(files.client).not.toContain(
      "operations['login']['responses'][200]['content']['application/json']",
    )

    expect(files.client).toContain('export interface GetExamplesSearchOptions {')
    expect(files.client).toContain("query: operations['search_examples']['parameters']['query']")
    expect(files.client).toContain('): Promise<ExampleSearchResponse> {')

    expect(files.client).toContain('export interface GetCatalogItemOptions {')
    expect(files.client).toContain("path: operations['get_item']['parameters']['path']")
    expect(files.client).toContain('): Promise<CatalogItemResponse> {')

    expect(files.client).toContain('export interface PostOrderReviewsOptions {')
    expect(files.client).toContain("path: operations['create_review']['parameters']['path']")
    expect(files.client).toContain("query?: operations['create_review']['parameters']['query']")
    expect(files.client).toContain('body: ReviewRequest')
    expect(files.client).toContain('): Promise<ReviewResponse> {')

    expect(files.client).toContain('export interface PostAuthLogoutOptions {')
    expect(files.client).toContain('return requestVoid(')
    expect(files.client).toContain('searchParams: buildSearchParams(options.query)')
  })
})

function createSpec(): Parameters<typeof renderGeneratedArtifacts>[0] {
  return {
    components: {
      schemas: {
        AuthResponse: {
          properties: {
            token: {
              type: 'string',
            },
          },
          type: 'object',
        },
        CatalogItemResponse: {
          properties: {
            item_id: {
              type: 'string',
            },
          },
          type: 'object',
        },
        ExampleSearchResponse: {
          properties: {
            page: {
              type: 'integer',
            },
          },
          type: 'object',
        },
        LoginRequest: {
          properties: {
            identifier: {
              type: 'string',
            },
          },
          type: 'object',
        },
        ReviewRequest: {
          properties: {
            rating: {
              type: 'integer',
            },
          },
          type: 'object',
        },
        ReviewResponse: {
          properties: {
            review_id: {
              type: 'string',
            },
          },
          type: 'object',
        },
      },
    },
    paths: {
      '/api/auth/login': {
        post: {
          operationId: 'login',
          requestBody: {
            content: {
              'application/json': {
                schema: {
                  $ref: '#/components/schemas/LoginRequest',
                },
              },
            },
          },
          responses: {
            200: {
              content: {
                'application/json': {
                  schema: {
                    $ref: '#/components/schemas/AuthResponse',
                  },
                },
              },
            },
          },
          tags: ['auth'],
        },
      },
      '/api/auth/logout': {
        post: {
          operationId: 'logout',
          responses: {
            204: {
              description: 'Logged out',
            },
          },
          tags: ['auth'],
        },
      },
      '/api/catalog/items/{item_id}': {
        get: {
          operationId: 'get_item',
          parameters: [
            {
              in: 'path',
              name: 'item_id',
              required: true,
              schema: {
                type: 'string',
              },
            },
          ],
          responses: {
            200: {
              content: {
                'application/json': {
                  schema: {
                    $ref: '#/components/schemas/CatalogItemResponse',
                  },
                },
              },
            },
          },
          tags: ['catalog'],
        },
      },
      '/api/examples/search': {
        get: {
          operationId: 'search_examples',
          parameters: [
            {
              in: 'query',
              name: 'page',
              required: true,
              schema: {
                type: 'integer',
              },
            },
          ],
          responses: {
            200: {
              content: {
                'application/json': {
                  schema: {
                    $ref: '#/components/schemas/ExampleSearchResponse',
                  },
                },
              },
            },
          },
          tags: ['examples'],
        },
      },
      '/api/orders/{order_id}/reviews': {
        post: {
          operationId: 'create_review',
          parameters: [
            {
              in: 'path',
              name: 'order_id',
              required: true,
              schema: {
                type: 'string',
              },
            },
            {
              in: 'query',
              name: 'notify',
              required: false,
              schema: {
                type: 'boolean',
              },
            },
          ],
          requestBody: {
            content: {
              'application/json': {
                schema: {
                  $ref: '#/components/schemas/ReviewRequest',
                },
              },
            },
          },
          responses: {
            201: {
              content: {
                'application/json': {
                  schema: {
                    $ref: '#/components/schemas/ReviewResponse',
                  },
                },
              },
            },
          },
          tags: ['orders'],
        },
      },
    },
  }
}

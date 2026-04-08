import { existsSync, readFileSync } from 'node:fs'
import { resolve } from 'node:path'

import * as ts from 'typescript'
import { describe, expect, it } from 'vite-plus/test'

import { renderGeneratedArtifacts } from './vite-plugin-openapi-codegen'

describe('vite-plugin-openapi-codegen', () => {
  it('generates path builders and typed request client functions', () => {
    const files = renderGeneratedArtifacts(createSpec(), {})
    const normalizedApi = normalizeGeneratedSource(files.api)
    const normalizedTypes = normalizeGeneratedSource(files.types)
    const normalizedClient = normalizeGeneratedSource(files.client)

    expect(normalizedApi).toContain('export function postAuthLogin(): string {')
    expect(normalizedApi).toContain('export function getCatalogItem(')
    expect(normalizedApi).toContain('export function getExamplesSearch(): string {')
    expect(files.api).not.toContain('buildQuery')

    expect(normalizedTypes).toContain("import type { components } from './api-types'")
    expect(normalizedTypes).toContain(
      "export type LoginRequest = components['schemas']['LoginRequest']",
    )
    expect(normalizedTypes).toContain(
      "export type ReviewResponse = components['schemas']['ReviewResponse']",
    )

    expect(normalizedClient).toContain(
      "import type { ApiRequestOptions } from '#/integrations/http'",
    )
    expect(normalizedClient).toContain(
      "import { requestJson, requestVoid } from '#/integrations/http'",
    )
    expect(normalizedClient).toContain('AuthResponse')
    expect(normalizedClient).toContain('CatalogItemResponse')
    expect(normalizedClient).toContain('ExampleSearchResponse')
    expect(normalizedClient).toContain('LoginRequest')
    expect(normalizedClient).toContain('ReviewRequest')
    expect(normalizedClient).toContain('ReviewResponse')
    expect(normalizedClient).toContain("from './types'")
    expect(normalizedClient).toContain("from './api'")

    expect(normalizedClient).toContain('export interface PostAuthLoginOptions {')
    expect(normalizedClient).toContain('query?: never')
    expect(normalizedClient).toContain('path?: never')
    expect(normalizedClient).toContain('body: LoginRequest')
    expect(normalizedClient).toContain('signal?: AbortSignal')
    expect(normalizedClient).toContain('): Promise<AuthResponse> {')
    expect(files.client).not.toContain(
      "operations['login']['requestBody']['content']['application/json']",
    )
    expect(files.client).not.toContain(
      "operations['login']['responses'][200]['content']['application/json']",
    )

    expect(normalizedClient).toContain('export interface GetExamplesSearchOptions {')
    expect(normalizedClient).toContain('query: ExampleSearchQuery')
    expect(normalizedClient).toContain('): Promise<ExampleSearchResponse> {')

    expect(normalizedClient).toContain('export interface GetCatalogItemOptions {')
    expect(normalizedClient).toContain('path: CatalogItemPath')
    expect(normalizedClient).toContain('): Promise<CatalogItemResponse> {')

    expect(normalizedClient).toContain('export interface PostOrderReviewsOptions {')
    expect(normalizedClient).toContain('path: OrderPath')
    expect(normalizedClient).toContain("query?: operations['create_review']['parameters']['query']")
    expect(normalizedClient).toContain('body: ReviewRequest')
    expect(normalizedClient).toContain('): Promise<ReviewResponse> {')

    expect(normalizedApi).toContain('params: CatalogItemPath')
    expect(normalizedApi).toContain("from './types'")

    expect(normalizedClient).toContain('export interface PostAuthLogoutOptions {')
    expect(normalizedClient).toContain('return requestVoid(')
    expect(normalizedClient).toContain('searchParams: buildSearchParams(options.query)')

    expectValidTypeScript(files.types, 'types.ts')
    expectValidTypeScript(files.api, 'api.ts')
    expectValidTypeScript(files.client, 'client.ts')
  })

  it('renders legacy aliases in generated types barrel', () => {
    const files = renderGeneratedArtifacts(createSpec(), {
      legacyAliases: {
        MenuItemPath: 'CatalogItemPath',
        MenuItemsQuery: 'CatalogItemsQuery',
      },
    })
    const normalizedTypes = normalizeGeneratedSource(files.types)

    expect(normalizedTypes).toContain('// Legacy aliases')
    expect(normalizedTypes).toContain('export type MenuItemPath = CatalogItemPath')
    expect(normalizedTypes).toContain('export type MenuItemsQuery = CatalogItemsQuery')
    expectValidTypeScript(files.types, 'types.ts')
  })

  it('maintains proper module boundaries', () => {
    const pluginSourceText = readFileSync(
      resolve(import.meta.dirname, './vite-plugin-openapi-codegen.ts'),
      'utf-8',
    )
    const astSourceText = readFileSync(
      resolve(import.meta.dirname, './openapi-codegen-ast.ts'),
      'utf-8',
    )
    const normalizationSourceText = readFileSync(
      resolve(import.meta.dirname, './openapi-codegen-normalization.ts'),
      'utf-8',
    )

    // Plugin entry imports from ast and normalization directly
    expect(pluginSourceText).toContain("from './openapi-codegen-ast.ts'")
    expect(pluginSourceText).toContain("from './openapi-codegen-normalization.ts'")

    // Plugin entry does NOT contain AST internals
    expect(pluginSourceText).not.toContain("import * as ts from 'typescript'")
    expect(pluginSourceText).not.toContain('function printGeneratedFile(')
    expect(pluginSourceText).not.toContain('function createBuildSearchParamsFunction(')

    // AST module imports from normalization for shared types
    expect(astSourceText).toContain("from './openapi-codegen-normalization.ts'")

    // Normalization exports key public functions
    expect(normalizationSourceText).toContain(
      'export function buildClientRenderModelFromOperations(',
    )
    expect(normalizationSourceText).toContain('export function collectOperations(')
    expect(normalizationSourceText).toContain('export function warnOnParameterLocationMismatch(')

    // Deleted files no longer exist
    expect(existsSync(resolve(import.meta.dirname, './openapi-codegen-ast-entry.ts'))).toBe(false)
    expect(existsSync(resolve(import.meta.dirname, './openapi-codegen-ast-adapters.ts'))).toBe(
      false,
    )
    expect(existsSync(resolve(import.meta.dirname, './openapi-codegen-ast-models.ts'))).toBe(false)
  })
})

function expectValidTypeScript(sourceText: string, fileName: string) {
  const result = ts.transpileModule(sourceText, {
    compilerOptions: {
      target: ts.ScriptTarget.ESNext,
    },
    fileName,
    reportDiagnostics: true,
  })

  const diagnostics = result.diagnostics?.filter(
    (diagnostic) => diagnostic.category === ts.DiagnosticCategory.Error,
  )
  expect(diagnostics ?? []).toHaveLength(0)
}

function normalizeGeneratedSource(sourceText: string): string {
  return sourceText.replaceAll('"', "'").replaceAll(';', '').replace(/\s+/g, ' ').trim()
}

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
        CatalogItemPath: {
          properties: {
            item_id: {
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
        ExampleSearchQuery: {
          properties: {
            page: {
              type: 'integer',
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
        OrderPath: {
          properties: {
            order_id: {
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

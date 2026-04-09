import { renderGeneratedArtifacts } from 'vite-plugin-openapi-codegen'
import { describe, expect, it } from 'vite-plus/test'

describe('vite-plugin-openapi-codegen package integration', () => {
  it('renders generated artifacts from the published package', () => {
    const files = renderGeneratedArtifacts(createSpec(), {
      httpClient: {
        module: '#/integrations/http',
        jsonFunction: 'requestJson',
        voidFunction: 'requestVoid',
        requestOptionsType: 'ApiRequestOptions',
        omitKeys: ['json', 'method', 'searchParams', 'signal'],
      },
    })

    expect(normalizeGeneratedSource(files.api)).toContain(
      'export function postAuthLogin(): string {',
    )
    expect(normalizeGeneratedSource(files.client)).toContain(
      "import { requestJson, requestVoid } from '#/integrations/http'",
    )
    expect(normalizeGeneratedSource(files.types)).toContain(
      "export type LoginRequest = components['schemas']['LoginRequest']",
    )
  })
})

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
        LoginRequest: {
          properties: {
            identifier: {
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
    },
  }
}

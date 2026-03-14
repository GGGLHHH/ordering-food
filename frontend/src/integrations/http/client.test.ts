// @vitest-environment jsdom

import { afterEach, describe, expect, it, vi } from 'vite-plus/test'
import { createApiClient } from './client'
import { ApiError } from './error'

const BASE_URL = 'http://localhost/api'

describe('api client', () => {
  afterEach(() => {
    vi.restoreAllMocks()
    window.history.replaceState({}, '', '/')
  })

  it('parses json responses', async () => {
    const fetchMock = vi.fn(async () =>
      createJsonResponse({
        ok: true,
      }),
    )
    const client = createApiClient({
      baseUrl: BASE_URL,
      fetch: fetchMock as typeof fetch,
    })

    await expect(client.requestJson<{ ok: boolean }>('examples/search?page=1')).resolves.toEqual({
      ok: true,
    })
  })

  it('handles void responses', async () => {
    const fetchMock = vi.fn(async () => new Response(null, { status: 204 }))
    const client = createApiClient({
      baseUrl: BASE_URL,
      fetch: fetchMock as typeof fetch,
    })

    await expect(
      client.requestVoid('auth/logout', {
        authMode: 'none',
        method: 'POST',
      }),
    ).resolves.toBeUndefined()
  })

  it('normalizes backend error envelopes', async () => {
    const fetchMock = vi.fn(async () =>
      createJsonResponse(
        {
          code: 'validation_error',
          details: {
            fields: [],
          },
          message: 'request body failed validation',
          request_id: 'request-123',
        },
        422,
      ),
    )
    const client = createApiClient({
      baseUrl: BASE_URL,
      fetch: fetchMock as typeof fetch,
    })

    await expect(client.requestJson('examples/echo', { method: 'POST' })).rejects.toMatchObject({
      code: 'validation_error',
      details: {
        fields: [],
      },
      isRetryable: false,
      requestId: 'request-123',
      status: 422,
    })
  })

  it('refreshes once and retries the original request', async () => {
    const fetchMock = vi
      .fn<typeof fetch>()
      .mockImplementationOnce(async () => createJsonResponse({ code: 'unauthorized' }, 401))
      .mockImplementationOnce(async () => createJsonResponse({ user_id: 'u-1', expires_in: 900 }))
      .mockImplementationOnce(async () => createJsonResponse({ display_name: 'Ada' }))
    const client = createApiClient({
      baseUrl: BASE_URL,
      fetch: fetchMock as typeof fetch,
    })

    await expect(
      client.requestJson<{ display_name: string }>('auth/me', {
        authMode: 'optional',
      }),
    ).resolves.toEqual({ display_name: 'Ada' })
    expect(fetchMock).toHaveBeenCalledTimes(3)
    expect(getRequestUrl(fetchMock.mock.calls[1]?.[0])).toBe(`${BASE_URL}/auth/refresh`)
  })

  it('deduplicates concurrent refresh requests', async () => {
    let refreshCalls = 0
    let meCalls = 0
    const fetchMock = vi.fn<typeof fetch>(async (input) => {
      const url = getRequestUrl(input)
      if (url.endsWith('/auth/refresh')) {
        refreshCalls += 1
        return createJsonResponse({ user_id: 'u-1', expires_in: 900 })
      }

      if (url.endsWith('/auth/me')) {
        meCalls += 1
      }

      if (meCalls <= 2) {
        return createJsonResponse({ code: 'unauthorized' }, 401)
      }

      return createJsonResponse({ display_name: 'Grace' })
    })
    const client = createApiClient({
      baseUrl: BASE_URL,
      fetch: fetchMock as typeof fetch,
    })

    await Promise.all([
      client.requestJson('auth/me', { authMode: 'optional' }),
      client.requestJson('auth/me', { authMode: 'optional' }),
    ])

    expect(refreshCalls).toBe(1)
    expect(meCalls).toBe(4)
  })

  it('does not refresh login or logout requests', async () => {
    const fetchMock = vi.fn<typeof fetch>().mockImplementation(async (input) => {
      const url = getRequestUrl(input)
      if (url.endsWith('/auth/refresh')) {
        return createJsonResponse({}, 200)
      }
      return createJsonResponse({ code: 'unauthorized', message: 'invalid credentials' }, 401)
    })
    const client = createApiClient({
      baseUrl: BASE_URL,
      fetch: fetchMock as typeof fetch,
    })

    await expect(
      client.requestJson('auth/login', {
        authMode: 'none',
        method: 'POST',
      }),
    ).rejects.toBeInstanceOf(ApiError)
    await expect(
      client.requestVoid('auth/logout', {
        authMode: 'none',
        method: 'POST',
      }),
    ).rejects.toBeInstanceOf(ApiError)

    expect(fetchMock.mock.calls.some(([input]) => String(input).endsWith('/auth/refresh'))).toBe(
      false,
    )
  })

  it('keeps optional auth failures local when refresh fails', async () => {
    const redirectSpy = vi.fn()
    const fetchMock = vi
      .fn<typeof fetch>()
      .mockImplementationOnce(async () => createJsonResponse({ code: 'unauthorized' }, 401))
      .mockImplementationOnce(async () => createJsonResponse({ code: 'unauthorized' }, 401))
    const client = createApiClient({
      baseUrl: BASE_URL,
      fetch: fetchMock as typeof fetch,
      onAuthRedirect: redirectSpy,
    })

    await expect(
      client.requestJson('auth/me', {
        authMode: 'optional',
      }),
    ).rejects.toMatchObject({
      code: 'unauthorized',
      isUnauthorized: true,
      status: 401,
    })
    expect(redirectSpy).not.toHaveBeenCalled()
  })

  it('redirects required auth failures to the login page', async () => {
    const redirectSpy = vi.fn()
    const fetchMock = vi
      .fn<typeof fetch>()
      .mockImplementationOnce(async () => createJsonResponse({ code: 'unauthorized' }, 401))
      .mockImplementationOnce(async () => createJsonResponse({ code: 'unauthorized' }, 401))
    const client = createApiClient({
      baseUrl: BASE_URL,
      fetch: fetchMock as typeof fetch,
      onAuthRedirect: redirectSpy,
    })
    window.history.replaceState({}, '', '/account/orders?tab=active')

    await expect(
      client.requestJson('auth/me', {
        authMode: 'required',
      }),
    ).rejects.toMatchObject({
      code: 'unauthorized',
      isUnauthorized: true,
      status: 401,
    })
    expect(redirectSpy).toHaveBeenCalledWith('/login?redirect=%2Faccount%2Forders%3Ftab%3Dactive')
  })

  it('passes abort signals to fetch', async () => {
    const controller = new AbortController()
    let receivedSignal: AbortSignal | undefined
    const fetchMock = vi.fn<typeof fetch>(async (input, init) => {
      return await new Promise<Response>((_resolve, reject) => {
        receivedSignal = getRequestSignal(input, init)
        receivedSignal?.addEventListener('abort', () => {
          reject(new DOMException('The operation was aborted.', 'AbortError'))
        })
      })
    })
    const client = createApiClient({
      baseUrl: BASE_URL,
      fetch: fetchMock as typeof fetch,
    })

    const request = client.requestJson('examples/search', {
      authMode: 'none',
      signal: controller.signal,
    })

    await vi.waitFor(() => {
      expect(receivedSignal).toBeDefined()
    })
    controller.abort()

    await expect(request).rejects.toMatchObject({
      name: 'AbortError',
    })
  })
})

function createJsonResponse(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    headers: {
      'content-type': 'application/json',
    },
    status,
  })
}

function getRequestSignal(input: RequestInfo | URL, init?: RequestInit) {
  if (input instanceof Request) {
    return input.signal ?? undefined
  }

  return init?.signal ?? undefined
}

function getRequestUrl(input: RequestInfo | URL | undefined) {
  if (!input) {
    return ''
  }

  if (input instanceof Request) {
    return input.url
  }

  return String(input)
}

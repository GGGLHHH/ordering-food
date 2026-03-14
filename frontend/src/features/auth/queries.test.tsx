// @vitest-environment jsdom

import { QueryClientProvider } from '@tanstack/react-query'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vite-plus/test'

import { ApiError } from '#/integrations/http'
import { createAppQueryClient, shouldRetryQuery } from '#/integrations/tanstack-query/query-client'

import { authKeys, useAuthSessionQuery, useLogoutMutation } from './queries'

const navigateSpy = vi.fn()

vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => navigateSpy,
}))

describe('auth query integration', () => {
  afterEach(() => {
    vi.restoreAllMocks()
    navigateSpy.mockReset()
    window.history.replaceState({}, '', '/')
  })

  it('retries only retryable API errors and keeps mutation retries disabled', () => {
    const queryClient = createAppQueryClient()

    expect(
      shouldRetryQuery(
        0,
        new ApiError({
          code: 'network_error',
          isRetryable: true,
          message: 'Network failed',
          status: null,
        }),
      ),
    ).toBe(true)
    expect(
      shouldRetryQuery(
        0,
        new ApiError({
          code: 'unauthorized',
          isRetryable: false,
          message: 'Unauthorized',
          status: 401,
        }),
      ),
    ).toBe(false)
    expect(queryClient.getDefaultOptions().mutations?.retry).toBe(0)
  })

  it('treats public auth/me probes as anonymous instead of redirecting', async () => {
    const fetchMock = vi
      .fn<typeof fetch>()
      .mockImplementationOnce(async () => createJsonResponse({ code: 'unauthorized' }, 401))
      .mockImplementationOnce(async () => createJsonResponse({ code: 'unauthorized' }, 401))
    vi.stubGlobal('fetch', fetchMock)
    const queryClient = createAppQueryClient()

    render(
      <QueryClientProvider client={queryClient}>
        <SessionProbe />
      </QueryClientProvider>,
    )

    await waitFor(() => {
      expect(screen.getByText('anonymous')).not.toBeNull()
    })
    expect(window.location.pathname).toBe('/')
  })

  it('re-fetches auth/me on a fresh mount even when the placeholder is anonymous', async () => {
    const fetchMock = vi.fn<typeof fetch>(async (input) => {
      const url = getRequestUrl(input)
      if (url.endsWith('/auth/me')) {
        return createJsonResponse({
          display_name: 'PW Debug',
          status: 'active',
          user_id: 'user-1',
        })
      }

      return createJsonResponse({ code: 'not_found' }, 404)
    })
    vi.stubGlobal('fetch', fetchMock)
    const queryClient = createAppQueryClient()

    render(
      <QueryClientProvider client={queryClient}>
        <SessionProbe />
      </QueryClientProvider>,
    )

    expect(screen.getAllByText('anonymous').length).toBeGreaterThan(0)

    await waitFor(() => {
      expect(screen.getByText('PW Debug')).not.toBeNull()
    })

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(getRequestUrl(fetchMock.mock.calls[0]?.[0])).toContain('/api/auth/me')
  })

  it('clears auth cache and navigates to login after logout', async () => {
    const fetchMock = vi.fn<typeof fetch>(async (input) => {
      const url = getRequestUrl(input)
      if (url.endsWith('/auth/logout')) {
        return new Response(null, { status: 204 })
      }
      return createJsonResponse({ code: 'not_found' }, 404)
    })
    vi.stubGlobal('fetch', fetchMock)
    const queryClient = createAppQueryClient()
    queryClient.setQueryData(authKeys.me(), {
      display_name: 'Ada',
      status: 'active',
      user_id: 'user-1',
    })

    render(
      <QueryClientProvider client={queryClient}>
        <LogoutProbe />
      </QueryClientProvider>,
    )

    fireEvent.click(screen.getByRole('button', { name: 'logout' }))

    await waitFor(() => {
      expect(navigateSpy).toHaveBeenCalledWith({
        to: '/login',
      })
    })
    expect(queryClient.getQueryData(authKeys.me())).toBeUndefined()
  })
})

function SessionProbe() {
  const query = useAuthSessionQuery()
  return <p>{query.data?.display_name ?? 'anonymous'}</p>
}

function LogoutProbe() {
  const logoutMutation = useLogoutMutation()

  return (
    <button
      type="button"
      onClick={() => {
        void logoutMutation.mutateAsync()
      }}
    >
      logout
    </button>
  )
}

function createJsonResponse(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    headers: {
      'content-type': 'application/json',
    },
    status,
  })
}

function getRequestUrl(input: RequestInfo | URL) {
  if (input instanceof Request) {
    return input.url
  }

  return String(input)
}

// @vitest-environment jsdom

import { QueryClientProvider } from '@tanstack/react-query'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { createAppQueryClient } from '#/integrations/tanstack-query/query-client'
import { LoginForm } from './login-form'

vi.mock('@tanstack/react-router', () => ({
  Link: ({ children }: { children?: ReactNode }) => <span>{children}</span>,
  useNavigate: () => vi.fn(),
}))

describe('login form', () => {
  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('logs in, refetches auth/me, and redirects to the requested page', async () => {
    const redirectSpy = vi.fn()
    const fetchMock = vi.fn<typeof fetch>(async (input) => {
      const url = getRequestUrl(input)
      if (url.endsWith('/auth/login')) {
        return createJsonResponse({
          expires_in: 900,
          user_id: 'user-1',
        })
      }

      if (url.endsWith('/auth/me')) {
        return createJsonResponse({
          display_name: 'Ada Lovelace',
          status: 'active',
          user_id: 'user-1',
        })
      }

      return createJsonResponse({}, 404)
    })
    vi.stubGlobal('fetch', fetchMock)
    const queryClient = createAppQueryClient()

    render(
      <QueryClientProvider client={queryClient}>
        <LoginForm onSuccessRedirect={redirectSpy} redirectTo="/about" />
      </QueryClientProvider>,
    )

    fireEvent.change(screen.getByLabelText('账号'), {
      target: {
        value: 'ada@example.com',
      },
    })
    fireEvent.change(screen.getByLabelText('密码'), {
      target: {
        value: 'secret',
      },
    })
    fireEvent.click(screen.getByRole('button', { name: '登录' }))

    await waitFor(() => {
      expect(redirectSpy).toHaveBeenCalledWith('/about')
    })
    const requestUrls = fetchMock.mock.calls.map(([input]) => getRequestUrl(input))

    expect(requestUrls.some((url) => url.includes('/api/auth/login'))).toBe(true)
    expect(requestUrls.filter((url) => url.includes('/api/auth/me')).length).toBeGreaterThan(0)
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

function getRequestUrl(input: RequestInfo | URL | undefined) {
  if (!input) {
    return ''
  }

  if (input instanceof Request) {
    return input.url
  }

  return String(input)
}

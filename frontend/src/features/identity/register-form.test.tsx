// @vitest-environment jsdom

import { QueryClientProvider } from '@tanstack/react-query'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vite-plus/test'
import { createAppQueryClient } from '#/integrations/tanstack-query/query-client'
import { RegisterForm } from './register-form'

vi.mock('@tanstack/react-router', () => ({
  Link: ({ children }: { children?: ReactNode }) => <span>{children}</span>,
}))

describe('register form', () => {
  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('creates a user and redirects back to login with the identifier', async () => {
    const redirectSpy = vi.fn()
    const fetchMock = vi.fn<typeof fetch>(async (input) => {
      const url = getRequestUrl(input)
      if (url.endsWith('/identity/users')) {
        return createJsonResponse(
          {
            created_at: '2026-03-12T00:00:00Z',
            deleted_at: null,
            identities: [
              {
                bound_at: '2026-03-12T00:00:00Z',
                identifier_normalized: 'ada@example.com',
                identity_type: 'email',
              },
            ],
            profile: {
              avatar_url: null,
              display_name: 'Ada',
              family_name: null,
              given_name: null,
            },
            status: 'active',
            updated_at: '2026-03-12T00:00:00Z',
            user_id: 'user-1',
          },
          201,
        )
      }

      return createJsonResponse({}, 404)
    })
    vi.stubGlobal('fetch', fetchMock)
    const queryClient = createAppQueryClient()

    render(
      <QueryClientProvider client={queryClient}>
        <RegisterForm onSuccessRedirect={redirectSpy} redirectTo="/checkout" />
      </QueryClientProvider>,
    )

    fireEvent.change(screen.getByLabelText('显示名称'), {
      target: {
        value: 'Ada',
      },
    })
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
    fireEvent.click(screen.getByRole('button', { name: '注册' }))

    await waitFor(() => {
      expect(redirectSpy).toHaveBeenCalledWith('ada@example.com')
    })

    const requestUrls = fetchMock.mock.calls.map(([input]) => getRequestUrl(input))
    expect(requestUrls.some((url) => url.includes('/api/identity/users'))).toBe(true)
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

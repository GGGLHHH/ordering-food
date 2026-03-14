import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { z } from 'zod'

import { LoginForm } from '#/features/auth/login-form'

const loginSearchSchema = z.object({
  identifier: z.string().optional(),
  redirect: z.string().optional(),
})

export const Route = createFileRoute('/login')({
  component: LoginRoute,
  validateSearch: loginSearchSchema,
})

function LoginRoute() {
  const navigate = useNavigate()
  const { identifier, redirect } = Route.useSearch()

  return (
    <main className="page-wrap px-4 pt-14 pb-8">
      <LoginForm
        initialIdentifier={identifier}
        redirectTo={redirect}
        onSuccessRedirect={async (href) => {
          await navigate({
            to: href,
          })
        }}
      />
    </main>
  )
}

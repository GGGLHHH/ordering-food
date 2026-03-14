import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { z } from 'zod'

import { RegisterForm } from '#/features/identity/register-form'

const registerSearchSchema = z.object({
  redirect: z.string().optional(),
})

export const Route = createFileRoute('/register')({
  component: RegisterRoute,
  validateSearch: registerSearchSchema,
})

function RegisterRoute() {
  const navigate = useNavigate()
  const { redirect } = Route.useSearch()

  return (
    <main className="page-wrap px-4 pt-14 pb-8">
      <RegisterForm
        redirectTo={redirect}
        onSuccessRedirect={async (identifier) => {
          await navigate({
            search: {
              identifier,
              redirect,
            },
            to: '/login',
          })
        }}
      />
    </main>
  )
}

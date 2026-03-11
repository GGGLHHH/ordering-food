import { createFileRoute, redirect } from '@tanstack/react-router'

export const Route = createFileRoute('/demo/tanstack-query')({
  beforeLoad: () => {
    throw redirect({ to: '/' })
  },
})

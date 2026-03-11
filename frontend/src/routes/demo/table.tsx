import { createFileRoute, redirect } from '@tanstack/react-router'

export const Route = createFileRoute('/demo/table')({
  beforeLoad: () => {
    throw redirect({ to: '/' })
  },
})

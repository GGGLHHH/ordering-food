import { createFileRoute, redirect } from '@tanstack/react-router'

export const Route = createFileRoute('/demo/store')({
  beforeLoad: () => {
    throw redirect({ to: '/' })
  },
})

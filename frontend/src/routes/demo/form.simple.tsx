import { createFileRoute, redirect } from '@tanstack/react-router'

export const Route = createFileRoute('/demo/form/simple')({
  beforeLoad: () => {
    throw redirect({ to: '/' })
  },
})

import { createFileRoute, redirect } from '@tanstack/react-router'

export const Route = createFileRoute('/demo/form/address')({
  beforeLoad: () => {
    throw redirect({ to: '/' })
  },
})

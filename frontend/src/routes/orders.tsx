import { Outlet, createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/orders')({
  component: OrdersRoute,
})

function OrdersRoute() {
  return <Outlet />
}

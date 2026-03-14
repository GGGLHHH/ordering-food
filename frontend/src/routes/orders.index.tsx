import { createFileRoute } from '@tanstack/react-router'

import { OrderListPage } from '#/features/orders/order-list-page'

export const Route = createFileRoute('/orders/')({
  component: OrdersIndexRoute,
})

function OrdersIndexRoute() {
  return <OrderListPage />
}

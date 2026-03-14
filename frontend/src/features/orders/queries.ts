import { mutationOptions, queryOptions, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import type { PlaceOrderRequest } from '#/contracts/generated'

import { getOrder, listOrders, placeOrder } from './api'

export const orderKeys = {
  all: ['orders'] as const,
  list: () => [...orderKeys.all, 'list'] as const,
  detail: (orderId: string) => [...orderKeys.all, 'detail', orderId] as const,
}

export const orderQueries = {
  list: () =>
    queryOptions({
      queryFn: ({ signal }) => listOrders(signal),
      queryKey: orderKeys.list(),
      staleTime: 5_000,
    }),
  detail: (orderId: string) =>
    queryOptions({
      enabled: orderId.trim().length > 0,
      queryFn: ({ signal }) => getOrder(orderId, signal),
      queryKey: orderKeys.detail(orderId),
      staleTime: 5_000,
    }),
}

export const orderMutations = {
  place: () =>
    mutationOptions({
      mutationFn: (payload: PlaceOrderRequest) => placeOrder(payload),
      mutationKey: [...orderKeys.all, 'place'] as const,
    }),
}

export function useOrderQuery(orderId: string) {
  return useQuery(orderQueries.detail(orderId))
}

export function useOrdersQuery() {
  return useQuery(orderQueries.list())
}

export function usePlaceOrderMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    ...orderMutations.place(),
    onSuccess: async (order) => {
      await queryClient.invalidateQueries({
        queryKey: orderKeys.list(),
      })
      queryClient.setQueryData(orderKeys.detail(order.order_id), order)
    },
  })
}

const RECENT_ORDER_KEY = 'ordering-food:recent-order-id'

export function readRecentOrderId() {
  if (typeof window === 'undefined') {
    return null
  }

  return window.localStorage.getItem(RECENT_ORDER_KEY)
}

export function writeRecentOrderId(orderId: string) {
  if (typeof window === 'undefined') {
    return
  }

  window.localStorage.setItem(RECENT_ORDER_KEY, orderId)
}

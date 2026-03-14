import type { OrderListResponse, OrderResponse, PlaceOrderRequest } from '#/contracts/generated'
import { requestJson } from '#/integrations/http'

export function placeOrder(payload: PlaceOrderRequest, signal?: AbortSignal) {
  return requestJson<OrderResponse>('orders', {
    authMode: 'required',
    json: payload,
    method: 'POST',
    signal,
  })
}

export function getOrder(orderId: string, signal?: AbortSignal) {
  return requestJson<OrderResponse>(`orders/${orderId}`, {
    authMode: 'required',
    method: 'GET',
    signal,
  })
}

export function listOrders(signal?: AbortSignal) {
  return requestJson<OrderListResponse>('orders', {
    authMode: 'required',
    method: 'GET',
    signal,
  })
}

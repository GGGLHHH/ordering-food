import type {
  OrderListResponse,
  OrderPath,
  OrderResponse,
  PlaceOrderRequest,
} from '#/contracts/openapi/types'
import { requestJson } from '#/integrations/http'

export function placeOrder(payload: PlaceOrderRequest, signal?: AbortSignal) {
  return requestJson<OrderResponse>('orders', {
    authMode: 'required',
    json: payload,
    method: 'POST',
    signal,
  })
}

export function getOrder(path: OrderPath, signal?: AbortSignal) {
  return requestJson<OrderResponse>(`orders/${path.order_id}`, {
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

import { getOrder as getOrderPath, getOrders, postOrders } from '#/contracts/openapi/api'
import type {
  OrderListResponse,
  OrderPath,
  OrderResponse,
  PlaceOrderRequest,
} from '#/contracts/openapi/types'
import { requestJson } from '#/integrations/http'

export function placeOrder(payload: PlaceOrderRequest, signal?: AbortSignal) {
  return requestJson<OrderResponse>(postOrders(), {
    authMode: 'required',
    json: payload,
    method: 'POST',
    signal,
  })
}

export function getOrder(path: OrderPath, signal?: AbortSignal) {
  return requestJson<OrderResponse>(getOrderPath({ order_id: path.order_id }), {
    authMode: 'required',
    method: 'GET',
    signal,
  })
}

export function listOrders(signal?: AbortSignal) {
  return requestJson<OrderListResponse>(getOrders(), {
    authMode: 'required',
    method: 'GET',
    signal,
  })
}

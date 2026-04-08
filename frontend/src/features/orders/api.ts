import {
  getOrder as getOrderRequest,
  getOrders as getOrdersRequest,
  postOrders as postOrdersRequest,
} from '#/contracts/openapi/client'
import type { OrderPath, PlaceOrderRequest } from '#/contracts/openapi/types'

export function placeOrder(payload: PlaceOrderRequest, signal?: AbortSignal) {
  return postOrdersRequest(
    {
      body: payload,
      signal,
    },
    {
      authMode: 'required',
    },
  )
}

export function getOrder(path: OrderPath, signal?: AbortSignal) {
  return getOrderRequest(
    {
      path,
      signal,
    },
    {
      authMode: 'required',
    },
  )
}

export function listOrders(signal?: AbortSignal) {
  return getOrdersRequest(
    {
      signal,
    },
    {
      authMode: 'required',
    },
  )
}

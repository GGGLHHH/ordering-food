import type { paths } from './api-types'

/** Compile-time validated API path constants. If a path doesn't exist in the OpenAPI spec, TypeScript will error. */
function apiPath<P extends keyof paths>(path: P): P {
  return path
}

// Strip the /api/ prefix for use with requestJson (which prepends /api/ automatically)
function stripApiPrefix<P extends keyof paths>(path: P): string {
  const str = path as string
  return str.startsWith('/api/') ? str.slice(5) : str
}

// Auth
export const AUTH_LOGIN = stripApiPrefix(apiPath('/api/auth/login'))
export const AUTH_LOGOUT = stripApiPrefix(apiPath('/api/auth/logout'))
export const AUTH_ME = stripApiPrefix(apiPath('/api/auth/me'))
export const AUTH_REFRESH = stripApiPrefix(apiPath('/api/auth/refresh'))

// Catalog
export const CATALOG_CATEGORIES = stripApiPrefix(apiPath('/api/catalog/categories'))
export const CATALOG_ITEMS = stripApiPrefix(apiPath('/api/catalog/items'))
export const CATALOG_STORE = stripApiPrefix(apiPath('/api/catalog/store'))
export function catalogItemPath(itemId: string) {
  return `${stripApiPrefix(apiPath('/api/catalog/items/{item_id}')).replace('{item_id}', itemId)}`
}

// Identity
export const IDENTITY_USERS = stripApiPrefix(apiPath('/api/identity/users'))
export function identityUserPath(userId: string) {
  return `${stripApiPrefix(apiPath('/api/identity/users/{user_id}')).replace('{user_id}', userId)}`
}
export function identityUserDisablePath(userId: string) {
  return `${stripApiPrefix(apiPath('/api/identity/users/{user_id}/disable')).replace('{user_id}', userId)}`
}
export function identityUserIdentitiesPath(userId: string) {
  return `${stripApiPrefix(apiPath('/api/identity/users/{user_id}/identities')).replace('{user_id}', userId)}`
}
export function identityUserProfilePath(userId: string) {
  return `${stripApiPrefix(apiPath('/api/identity/users/{user_id}/profile')).replace('{user_id}', userId)}`
}
export function identityUserSoftDeletePath(userId: string) {
  return `${stripApiPrefix(apiPath('/api/identity/users/{user_id}/soft-delete')).replace('{user_id}', userId)}`
}

// Orders
export const ORDERS = stripApiPrefix(apiPath('/api/orders'))
export function orderPath(orderId: string) {
  return `${stripApiPrefix(apiPath('/api/orders/{order_id}')).replace('{order_id}', orderId)}`
}
export function orderAcceptPath(orderId: string) {
  return `${stripApiPrefix(apiPath('/api/orders/{order_id}/accept')).replace('{order_id}', orderId)}`
}
export function orderCancelPath(orderId: string) {
  return `${stripApiPrefix(apiPath('/api/orders/{order_id}/cancel')).replace('{order_id}', orderId)}`
}
export function orderCompletePath(orderId: string) {
  return `${stripApiPrefix(apiPath('/api/orders/{order_id}/complete')).replace('{order_id}', orderId)}`
}
export function orderReadyPath(orderId: string) {
  return `${stripApiPrefix(apiPath('/api/orders/{order_id}/ready')).replace('{order_id}', orderId)}`
}
export function orderRejectPath(orderId: string) {
  return `${stripApiPrefix(apiPath('/api/orders/{order_id}/reject')).replace('{order_id}', orderId)}`
}
export function orderStartPreparingPath(orderId: string) {
  return `${stripApiPrefix(apiPath('/api/orders/{order_id}/start-preparing')).replace('{order_id}', orderId)}`
}

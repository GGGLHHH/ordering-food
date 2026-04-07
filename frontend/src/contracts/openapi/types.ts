import type { components } from './api-types'

// Auth
export type AuthMeResponse = components['schemas']['AuthMeResponse']
export type AuthResponse = components['schemas']['AuthResponse']
export type LoginRequest = components['schemas']['LoginRequest']

// Catalog
export type CatalogCategoriesResponse = components['schemas']['CatalogCategoriesResponse']
export type CatalogCategoryResponse = components['schemas']['CatalogCategoryResponse']
export type CatalogItemPath = components['schemas']['CatalogItemPath']
export type CatalogItemResponse = components['schemas']['CatalogItemResponse']
export type CatalogItemsQuery = components['schemas']['CatalogItemsQuery']
export type CatalogItemsResponse = components['schemas']['CatalogItemsResponse']
export type CatalogStoreCatalogResponse = components['schemas']['CatalogStoreCatalogResponse']

// Identity
export type BindIdentityUserIdentityRequest =
  components['schemas']['BindIdentityUserIdentityRequest']
export type CreateIdentityUserIdentityRequest =
  components['schemas']['CreateIdentityUserIdentityRequest']
export type CreateIdentityUserRequest = components['schemas']['CreateIdentityUserRequest']
export type IdentityUserIdentityResponse = components['schemas']['IdentityUserIdentityResponse']
export type IdentityUserPath = components['schemas']['IdentityUserPath']
export type IdentityUserProfileResponse = components['schemas']['IdentityUserProfileResponse']
export type IdentityUserResponse = components['schemas']['IdentityUserResponse']
export type UpdateIdentityUserProfileRequest =
  components['schemas']['UpdateIdentityUserProfileRequest']

// Orders
export type OrderItemResponse = components['schemas']['OrderItemResponse']
export type OrderListItemResponse = components['schemas']['OrderListItemResponse']
export type OrderListResponse = components['schemas']['OrderListResponse']
export type OrderPath = components['schemas']['OrderPath']
export type OrderResponse = components['schemas']['OrderResponse']
export type PlaceOrderItemRequest = components['schemas']['PlaceOrderItemRequest']
export type PlaceOrderRequest = components['schemas']['PlaceOrderRequest']

// Fulfillment
export type FulfillmentOrderItemResponse = components['schemas']['FulfillmentOrderItemResponse']
export type FulfillmentOrderPath = components['schemas']['FulfillmentOrderPath']
export type FulfillmentOrderResponse = components['schemas']['FulfillmentOrderResponse']

// Examples
export type ExampleItemResponse = components['schemas']['ExampleItemResponse']
export type ExamplePayload = components['schemas']['ExamplePayload']
export type ExamplePayloadResponse = components['schemas']['ExamplePayloadResponse']
export type ExampleSearchResponse = components['schemas']['ExampleSearchResponse']

// Error
export type ErrorDetails = components['schemas']['ErrorDetails']
export type ErrorEnvelope = components['schemas']['ErrorEnvelope']
export type FieldIssue = components['schemas']['FieldIssue']
export type FieldLocation = components['schemas']['FieldLocation']

// Infrastructure
export type DependencyChecks = components['schemas']['DependencyChecks']
export type LiveResponse = components['schemas']['LiveResponse']
export type PageMeta = components['schemas']['PageMeta']
export type ReadyResponse = components['schemas']['ReadyResponse']

// --- Legacy aliases (Menu* → Catalog*) ---
export type MenuStoreResponse = CatalogStoreCatalogResponse
export type MenuCategoriesResponse = CatalogCategoriesResponse
export type MenuCategoryResponse = CatalogCategoryResponse
export type MenuItemPath = CatalogItemPath
export type MenuItemResponse = CatalogItemResponse
export type MenuItemsQuery = CatalogItemsQuery
export type MenuItemsResponse = CatalogItemsResponse

// --- Path/query param types (not in OpenAPI schemas, inlined in operations) ---
export interface ExampleItemPath {
  item_id: number
}
export interface ExampleSearchQuery {
  page: number
}

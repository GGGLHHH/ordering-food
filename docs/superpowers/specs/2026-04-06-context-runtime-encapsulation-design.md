# ContextRuntime Encapsulation & Dependency Cleanup

Date: 2026-04-06
Status: Approved

## Background

After completing V-1 through V-8 architecture fixes in the `feat/backend-ddd-phase-1-boundary-skeleton` branch, a fresh audit found 3 remaining violations (NV-1, NV-2, NV-3). This spec covers fixing all three.

## Violations

### NV-1 (Medium): ContextRuntime fields are `pub`

All 6 ContextRuntime structs expose their fields as `pub`, allowing external code to access internals directly. This is inconsistent with the already-fixed OrganizationModule field privatization (V-1).

**Affected files:**
- `crates/identity-integration/src/lib.rs` — `IdentityContextRuntime` (3 fields)
- `crates/organization-integration/src/lib.rs` — `OrganizationContextRuntime` (3 fields)
- `crates/access-integration/src/lib.rs` — `AccessContextRuntime` (2 fields)
- `crates/catalog-integration/src/lib.rs` — `CatalogContextRuntime` (1 field)
- `crates/ordering-integration/src/lib.rs` — `OrderingContextRuntime` (1 field)
- `crates/fulfillment-integration/src/lib.rs` — `FulfillmentContextRuntime` (2 fields)

**External call sites** (all in `apps/api/src/composition/contexts/`):
- `identity.rs`: `runtime.module.clone()`, `runtime.access_token_verifier.clone()`, `runtime.subject_lookup_gateway.clone()`
- `organization.rs`: `runtime.store_scope_gateway.clone()`, `runtime.brand_lookup_gateway.clone()`
- `access.rs`: `access_runtime.order_management_gateway.clone()`
- `catalog.rs`: `catalog_runtime.module.clone()`
- `ordering.rs`: `runtime.module`
- `fulfillment.rs`: `runtime.module`, `runtime.ordering_event_projector`

### NV-2 (Low): Unused cross-context dependency

`crates/fulfillment-infrastructure-sqlx/Cargo.toml` declares `ordering-food-ordering-published` in `[dependencies]` but no source file uses it. Infrastructure crates should not hold cross-context dependencies.

### NV-3 (Low): IdentityContextConfig fields are `pub`

`IdentityContextConfig` has a `new()` constructor but its 3 fields (`jwt_secret`, `access_token_ttl_seconds`, `refresh_token_ttl_seconds`) are still `pub`. Inconsistent encapsulation.

## Design

### NV-1: Unified accessor pattern

For each ContextRuntime, apply the same mechanical transformation:

```rust
// Before
#[derive(Clone)]
pub struct FooContextRuntime {
    pub field: Arc<FooModule>,
}

// After
#[derive(Clone)]
pub struct FooContextRuntime {
    field: Arc<FooModule>,
}

impl FooContextRuntime {
    pub fn field(&self) -> &Arc<FooModule> {
        &self.field
    }
}
```

The constructor functions (`build_*_context_runtime`) already create the struct internally, so they continue to work with private fields.

External call sites change from `runtime.field` to `runtime.field()` (for by-ref) or `runtime.field().clone()` (for owned clone).

**Field-by-field accessor table:**

| Runtime | Field | Accessor signature |
|---------|-------|--------------------|
| `IdentityContextRuntime` | `module` | `pub fn module(&self) -> &Arc<IdentityModule>` |
| | `access_token_verifier` | `pub fn access_token_verifier(&self) -> &Arc<dyn AccessTokenVerifier>` |
| | `subject_lookup_gateway` | `pub fn subject_lookup_gateway(&self) -> &Arc<dyn SubjectLookupGateway>` |
| `OrganizationContextRuntime` | `module` | `pub fn module(&self) -> &Arc<OrganizationModule>` |
| | `store_scope_gateway` | `pub fn store_scope_gateway(&self) -> &Arc<dyn StoreScopeGateway>` |
| | `brand_lookup_gateway` | `pub fn brand_lookup_gateway(&self) -> &Arc<dyn BrandLookupGateway>` |
| `AccessContextRuntime` | `service` | `pub fn service(&self) -> &Arc<AccessService>` |
| | `order_management_gateway` | `pub fn order_management_gateway(&self) -> &Arc<dyn OrderManagementAccessGateway>` |
| `CatalogContextRuntime` | `module` | `pub fn module(&self) -> &Arc<CatalogModule>` |
| `OrderingContextRuntime` | `module` | `pub fn module(&self) -> &Arc<OrderingModule>` |
| `FulfillmentContextRuntime` | `module` | `pub fn module(&self) -> &Arc<FulfillmentModule>` |
| | `ordering_event_projector` | `pub fn ordering_event_projector(&self) -> &OrderingEventProjector` |

### NV-2: Remove unused dependency

Delete the `ordering-food-ordering-published` line from `crates/fulfillment-infrastructure-sqlx/Cargo.toml`.

### NV-3: Privatize IdentityContextConfig fields

Remove `pub` from all 3 fields. The `new()` constructor and `#[derive(Clone)]` provide sufficient access. No accessor methods needed since config is only consumed at construction time.

### Architecture tests

Check existing architecture tests for patterns that match `pub` field access and update if needed.

## Execution Order

1. NV-2: Remove unused Cargo.toml dependency (standalone, no code changes)
2. NV-3: Privatize IdentityContextConfig fields (standalone, no external callers)
3. NV-1: Privatize all 6 ContextRuntime structs + add accessors + update call sites
4. Build + test verification

## Risk Assessment

All changes are mechanical field visibility + accessor additions. No behavioral changes. Risk: Low.

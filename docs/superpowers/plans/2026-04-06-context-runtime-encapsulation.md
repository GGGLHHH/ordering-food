# ContextRuntime Encapsulation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Privatize all 6 ContextRuntime struct fields, add accessor methods, remove unused dependency, and privatize IdentityContextConfig fields — completing architecture purity.

**Architecture:** Mechanical field visibility changes across 6 integration crates + API composition layer. Each ContextRuntime gets private fields with `pub fn field(&self) -> &T` accessors. External call sites switch from `runtime.field` to `runtime.field()` or `runtime.field().clone()`.

**Tech Stack:** Rust, hexagonal architecture, DDD bounded contexts

**Spec:** `docs/superpowers/specs/2026-04-06-context-runtime-encapsulation-design.md`

---

### Task 1: NV-2 — Remove unused dependency from fulfillment-infrastructure-sqlx

**Files:**
- Modify: `server/crates/fulfillment-infrastructure-sqlx/Cargo.toml`

- [ ] **Step 1: Remove the unused dependency**

In `server/crates/fulfillment-infrastructure-sqlx/Cargo.toml`, delete the line:
```toml
ordering-food-ordering-published = { path = "../ordering-published" }
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo build -p ordering-food-fulfillment-infrastructure-sqlx`
Expected: compiles with no errors

- [ ] **Step 3: Commit**

```bash
git add server/crates/fulfillment-infrastructure-sqlx/Cargo.toml
git commit -m "fix(fulfillment): remove unused ordering-published dependency from infrastructure"
```

---

### Task 2: NV-3 — Privatize IdentityContextConfig fields

**Files:**
- Modify: `server/crates/identity-integration/src/lib.rs:20-25`

- [ ] **Step 1: Remove `pub` from the 3 fields**

Change lines 20-25 from:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityContextConfig {
    pub jwt_secret: String,
    pub access_token_ttl_seconds: u64,
    pub refresh_token_ttl_seconds: u64,
}
```
To:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityContextConfig {
    jwt_secret: String,
    access_token_ttl_seconds: u64,
    refresh_token_ttl_seconds: u64,
}
```

No accessor methods are needed — the config is only consumed at construction time via `new()`, and derive macros (`Debug`, `Clone`, `PartialEq`, `Eq`) work with private fields.

- [ ] **Step 2: Verify it compiles**

Run: `cargo build --workspace`
Expected: compiles with no errors. The only place that constructs `IdentityContextConfig` is `build_identity_context_runtime` (same crate) and `IdentityContextConfig::new()` (constructor) — both work with private fields.

- [ ] **Step 3: Commit**

```bash
git add server/crates/identity-integration/src/lib.rs
git commit -m "fix(identity): privatize IdentityContextConfig fields"
```

---

### Task 3: NV-1a — Privatize IdentityContextRuntime

**Files:**
- Modify: `server/crates/identity-integration/src/lib.rs:41-46`
- Modify: `server/apps/api/src/composition/contexts/identity.rs:45-49`

- [ ] **Step 1: Privatize fields and add accessors**

In `server/crates/identity-integration/src/lib.rs`, change lines 41-46 from:
```rust
#[derive(Clone)]
pub struct IdentityContextRuntime {
    pub module: Arc<IdentityModule>,
    pub access_token_verifier: Arc<dyn AccessTokenVerifier>,
    pub subject_lookup_gateway: Arc<dyn SubjectLookupGateway>,
}
```
To:
```rust
#[derive(Clone)]
pub struct IdentityContextRuntime {
    module: Arc<IdentityModule>,
    access_token_verifier: Arc<dyn AccessTokenVerifier>,
    subject_lookup_gateway: Arc<dyn SubjectLookupGateway>,
}

impl IdentityContextRuntime {
    pub fn module(&self) -> &Arc<IdentityModule> {
        &self.module
    }

    pub fn access_token_verifier(&self) -> &Arc<dyn AccessTokenVerifier> {
        &self.access_token_verifier
    }

    pub fn subject_lookup_gateway(&self) -> &Arc<dyn SubjectLookupGateway> {
        &self.subject_lookup_gateway
    }
}
```

- [ ] **Step 2: Update the API composition call site**

In `server/apps/api/src/composition/contexts/identity.rs`, change lines 45-49 from:
```rust
            let module = runtime.module.clone();
            let token_verifier: Arc<dyn AccessTokenVerifier> =
                runtime.access_token_verifier.clone();
            let subject_lookup_gateway: Arc<dyn SubjectLookupGateway> =
                runtime.subject_lookup_gateway.clone();
```
To:
```rust
            let module = runtime.module().clone();
            let token_verifier: Arc<dyn AccessTokenVerifier> =
                runtime.access_token_verifier().clone();
            let subject_lookup_gateway: Arc<dyn SubjectLookupGateway> =
                runtime.subject_lookup_gateway().clone();
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build --workspace`
Expected: no errors

- [ ] **Step 4: Commit**

```bash
git add server/crates/identity-integration/src/lib.rs server/apps/api/src/composition/contexts/identity.rs
git commit -m "refactor(identity): privatize IdentityContextRuntime fields with accessors"
```

---

### Task 4: NV-1b — Privatize OrganizationContextRuntime

**Files:**
- Modify: `server/crates/organization-integration/src/lib.rs:20-25`
- Modify: `server/apps/api/src/composition/contexts/organization.rs:33,37`

- [ ] **Step 1: Privatize fields and add accessors**

In `server/crates/organization-integration/src/lib.rs`, change lines 20-25 from:
```rust
#[derive(Clone)]
pub struct OrganizationContextRuntime {
    pub module: Arc<OrganizationModule>,
    pub store_scope_gateway: Arc<dyn StoreScopeGateway>,
    pub brand_lookup_gateway: Arc<dyn BrandLookupGateway>,
}
```
To:
```rust
#[derive(Clone)]
pub struct OrganizationContextRuntime {
    module: Arc<OrganizationModule>,
    store_scope_gateway: Arc<dyn StoreScopeGateway>,
    brand_lookup_gateway: Arc<dyn BrandLookupGateway>,
}

impl OrganizationContextRuntime {
    pub fn module(&self) -> &Arc<OrganizationModule> {
        &self.module
    }

    pub fn store_scope_gateway(&self) -> &Arc<dyn StoreScopeGateway> {
        &self.store_scope_gateway
    }

    pub fn brand_lookup_gateway(&self) -> &Arc<dyn BrandLookupGateway> {
        &self.brand_lookup_gateway
    }
}
```

Note: `seed_default_organization` (same crate) accesses `runtime.module` directly — Rust allows private field access within the same crate, so no change needed there.

- [ ] **Step 2: Update the API composition call site**

In `server/apps/api/src/composition/contexts/organization.rs`, change lines 31-37 from:
```rust
            let runtime = build_organization_context_runtime(pg_pool, clock);
            capabilities.publish(
                ORGANIZATION_STORE_SCOPE_GATEWAY,
                runtime.store_scope_gateway.clone(),
            );
            capabilities.publish(
                ORGANIZATION_BRAND_LOOKUP_GATEWAY,
                runtime.brand_lookup_gateway.clone(),
            );
```
To:
```rust
            let runtime = build_organization_context_runtime(pg_pool, clock);
            capabilities.publish(
                ORGANIZATION_STORE_SCOPE_GATEWAY,
                runtime.store_scope_gateway().clone(),
            );
            capabilities.publish(
                ORGANIZATION_BRAND_LOOKUP_GATEWAY,
                runtime.brand_lookup_gateway().clone(),
            );
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build --workspace`
Expected: no errors

- [ ] **Step 4: Commit**

```bash
git add server/crates/organization-integration/src/lib.rs server/apps/api/src/composition/contexts/organization.rs
git commit -m "refactor(organization): privatize OrganizationContextRuntime fields with accessors"
```

---

### Task 5: NV-1c — Privatize AccessContextRuntime

**Files:**
- Modify: `server/crates/access-integration/src/lib.rs:13-17`
- Modify: `server/apps/api/src/composition/contexts/access.rs:54`

- [ ] **Step 1: Privatize fields and add accessors**

In `server/crates/access-integration/src/lib.rs`, change lines 13-17 from:
```rust
#[derive(Clone)]
pub struct AccessContextRuntime {
    pub service: Arc<AccessService>,
    pub order_management_gateway: Arc<dyn OrderManagementAccessGateway>,
}
```
To:
```rust
#[derive(Clone)]
pub struct AccessContextRuntime {
    service: Arc<AccessService>,
    order_management_gateway: Arc<dyn OrderManagementAccessGateway>,
}

impl AccessContextRuntime {
    pub fn service(&self) -> &Arc<AccessService> {
        &self.service
    }

    pub fn order_management_gateway(&self) -> &Arc<dyn OrderManagementAccessGateway> {
        &self.order_management_gateway
    }
}
```

- [ ] **Step 2: Update the API composition call site**

In `server/apps/api/src/composition/contexts/access.rs`, change line 54 from:
```rust
                access_runtime.order_management_gateway.clone();
```
To:
```rust
                access_runtime.order_management_gateway().clone();
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build --workspace`
Expected: no errors

- [ ] **Step 4: Commit**

```bash
git add server/crates/access-integration/src/lib.rs server/apps/api/src/composition/contexts/access.rs
git commit -m "refactor(access): privatize AccessContextRuntime fields with accessors"
```

---

### Task 6: NV-1d — Privatize CatalogContextRuntime

**Files:**
- Modify: `server/crates/catalog-integration/src/lib.rs:17-20`
- Modify: `server/apps/api/src/composition/contexts/catalog.rs:66`

- [ ] **Step 1: Privatize fields and add accessor**

In `server/crates/catalog-integration/src/lib.rs`, change lines 17-20 from:
```rust
#[derive(Clone)]
pub struct CatalogContextRuntime {
    pub module: Arc<CatalogModule>,
}
```
To:
```rust
#[derive(Clone)]
pub struct CatalogContextRuntime {
    module: Arc<CatalogModule>,
}

impl CatalogContextRuntime {
    pub fn module(&self) -> &Arc<CatalogModule> {
        &self.module
    }
}
```

Note: `seed_default_catalog` (same crate) accesses `runtime.module` directly — Rust allows private field access within the same crate, so no change needed.

- [ ] **Step 2: Update the API composition call site**

In `server/apps/api/src/composition/contexts/catalog.rs`, change line 66 from:
```rust
            let catalog_module = catalog_runtime.module.clone();
```
To:
```rust
            let catalog_module = catalog_runtime.module().clone();
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build --workspace`
Expected: no errors

- [ ] **Step 4: Commit**

```bash
git add server/crates/catalog-integration/src/lib.rs server/apps/api/src/composition/contexts/catalog.rs
git commit -m "refactor(catalog): privatize CatalogContextRuntime fields with accessors"
```

---

### Task 7: NV-1e — Privatize OrderingContextRuntime

**Files:**
- Modify: `server/crates/ordering-integration/src/lib.rs:9-12`
- Modify: `server/apps/api/src/composition/contexts/ordering.rs:35`

- [ ] **Step 1: Privatize fields and add accessor**

In `server/crates/ordering-integration/src/lib.rs`, change lines 9-12 from:
```rust
#[derive(Clone)]
pub struct OrderingContextRuntime {
    pub module: Arc<OrderingModule>,
}
```
To:
```rust
#[derive(Clone)]
pub struct OrderingContextRuntime {
    module: Arc<OrderingModule>,
}

impl OrderingContextRuntime {
    pub fn module(&self) -> &Arc<OrderingModule> {
        &self.module
    }
}
```

- [ ] **Step 2: Update the API composition call site (move → clone)**

In `server/apps/api/src/composition/contexts/ordering.rs`, change line 35 from:
```rust
            let module = runtime.module;
```
To:
```rust
            let module = runtime.module().clone();
```

**Important:** This is a **move-to-clone** migration. The original code moved the `Arc` out of the struct. With private fields, we must clone via the accessor instead.

- [ ] **Step 3: Verify it compiles**

Run: `cargo build --workspace`
Expected: no errors

- [ ] **Step 4: Commit**

```bash
git add server/crates/ordering-integration/src/lib.rs server/apps/api/src/composition/contexts/ordering.rs
git commit -m "refactor(ordering): privatize OrderingContextRuntime fields with accessors"
```

---

### Task 8: NV-1f — Privatize FulfillmentContextRuntime

**Files:**
- Modify: `server/crates/fulfillment-integration/src/lib.rs:19-23`
- Modify: `server/apps/api/src/composition/contexts/fulfillment.rs:60-61`

- [ ] **Step 1: Privatize fields and add accessors**

In `server/crates/fulfillment-integration/src/lib.rs`, change lines 19-23 from:
```rust
#[derive(Clone)]
pub struct FulfillmentContextRuntime {
    pub module: Arc<FulfillmentModule>,
    pub ordering_event_projector: OrderingEventProjector,
}
```
To:
```rust
#[derive(Clone)]
pub struct FulfillmentContextRuntime {
    module: Arc<FulfillmentModule>,
    ordering_event_projector: OrderingEventProjector,
}

impl FulfillmentContextRuntime {
    pub fn module(&self) -> &Arc<FulfillmentModule> {
        &self.module
    }

    pub fn ordering_event_projector(&self) -> &OrderingEventProjector {
        &self.ordering_event_projector
    }
}
```

- [ ] **Step 2: Update the API composition call site (move → clone)**

In `server/apps/api/src/composition/contexts/fulfillment.rs`, change lines 60-61 from:
```rust
            let module = runtime.module;
            let projector = runtime.ordering_event_projector;
```
To:
```rust
            let module = runtime.module().clone();
            let projector = runtime.ordering_event_projector().clone();
```

**Important:** Both fields are **move-to-clone** migrations. The original code moved values out of the struct. With private fields, we must clone via accessors instead.

- [ ] **Step 3: Verify it compiles**

Run: `cargo build --workspace`
Expected: no errors

- [ ] **Step 4: Commit**

```bash
git add server/crates/fulfillment-integration/src/lib.rs server/apps/api/src/composition/contexts/fulfillment.rs
git commit -m "refactor(fulfillment): privatize FulfillmentContextRuntime fields with accessors"
```

---

### Task 9: Full build + test verification

**Files:** None (verification only)

- [ ] **Step 1: Full workspace build**

Run: `cargo build --workspace`
Expected: compiles with 0 errors, 0 warnings

- [ ] **Step 2: Full test suite (excluding DB-dependent crates)**

Run:
```bash
cargo test --workspace \
  --exclude ordering-food-identity-infrastructure-sqlx \
  --exclude ordering-food-organization-infrastructure-sqlx \
  --exclude ordering-food-catalog-infrastructure-sqlx \
  --exclude ordering-food-ordering-infrastructure-sqlx \
  --exclude ordering-food-fulfillment-infrastructure-sqlx \
  --exclude ordering-food-database-infrastructure-sqlx \
  --exclude ordering-food-access-infrastructure-sqlx
```
Expected: all tests pass, 0 failures

- [ ] **Step 3: Verify no remaining `pub` fields on ContextRuntime structs**

Run: `grep -rn "pub \(module\|service\|access_token_verifier\|subject_lookup_gateway\|store_scope_gateway\|brand_lookup_gateway\|order_management_gateway\|ordering_event_projector\):" server/crates/*-integration/src/lib.rs`
Expected: no matches (all fields are now private)

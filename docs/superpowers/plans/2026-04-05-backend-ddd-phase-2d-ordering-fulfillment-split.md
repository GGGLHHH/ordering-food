# Backend DDD Phase 2D Ordering / Fulfillment Split Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the current mixed `order` context into `Ordering` and `Fulfillment` without losing existing behavior: `Ordering` owns order creation, commercial snapshot, and customer-side cancellation semantics; `Fulfillment` owns store-side workflow progression and staff-facing operational actions.

**Architecture:** This phase is a boundary redraw, not the final event-driven collaboration phase. `Ordering` becomes the owner of commercial order truth in `ordering` schema, while `Fulfillment` becomes the owner of store-side workflow truth in its own `fulfillment` schema. Before Phase 3 builds the event spine, only two temporary synchronous published exceptions are allowed: `Ordering -> Fulfillment` bootstrap for newly placed/cancelled orders, and `Fulfillment -> Ordering` commercial-state guard for workflow transitions that must immediately decide against current commercial state. No context may write another context's tables directly.

**Tech Stack:** Rust 2024, Cargo workspace, Tokio, Axum, SQLx, thiserror, async-trait, cargo test / cargo nextest, architecture tests based on manifest/source scanning, temporary synchronous published interface over in-process module wiring.

---

## Scope Check

This plan only covers the boundary split of the current `order` context.

It does **not** cover:

1. Outbox / dispatcher / projector implementation
2. Cross-context asynchronous published event network
3. Final elimination of the temporary synchronous `Ordering <-> Fulfillment` collaboration paths
4. Long-term projection rebuild / idempotency / event versioning governance

This plan should execute **after** the following earlier plans reach their minimum contract-ready state:

1. `Phase 2A Organization foundation`
   `Organization` must already define store / brand scope ownership and publish stable scope refs.
2. `Phase 2B Access + Identity purification`
   Staff authorization checks must no longer be hard-wired to legacy `authz` naming.
3. `Phase 2C Catalog migration`
   Item snapshot terminology must align to `Catalog`, even if the order snapshot still stores copied item name / price facts locally.

## Planned File Map

### Existing files to modify

- `server/Cargo.toml`
  Register new `ordering-*` and `fulfillment-infrastructure-sqlx` members, then retire old `order-*` members when the cutover completes.
- `server/apps/api/Cargo.toml`
  Add `ordering-*` / `fulfillment-*` dependencies and remove `ordering-food-order-*` dependencies after the final cutover.
- `server/apps/api/src/composition/contexts/mod.rs`
  Replace `order` registration module with `ordering` and `fulfillment`.
- `server/apps/api/src/routes/mod.rs`
  Replace `orders` route module export with `ordering` and `fulfillment`.
- `server/apps/api/src/ts_bindings.rs`
  Export new route contract types after the route split.
- `server/apps/api/tests/order_architecture.rs`
  Replace legacy `order`-focused app-shell guard with `ordering` / `fulfillment` rules.
- `server/crates/database-infrastructure-sqlx/src/bin/migration-info.rs`
  Register the new `fulfillment` context migration so migration ordering remains explicit and reviewable.
- `server/crates/ordering-published/src/lib.rs`
  Expand placeholder contracts into the minimum temporary synchronous published interfaces and stable external refs needed by `Fulfillment`.
- `server/crates/fulfillment-published/src/lib.rs`
  Expand placeholder contracts into fulfillment-side external refs and status view contracts.

### Existing files to retire after cutover

- `server/apps/api/src/composition/contexts/order.rs`
- `server/apps/api/src/routes/orders.rs`
- `server/crates/order-domain/Cargo.toml`
- `server/crates/order-domain/src/lib.rs`
- `server/crates/order-domain/src/customer_id.rs`
- `server/crates/order-domain/src/error.rs`
- `server/crates/order-domain/src/menu_item_id.rs`
- `server/crates/order-domain/src/order.rs`
- `server/crates/order-domain/src/order_id.rs`
- `server/crates/order-domain/src/order_item.rs`
- `server/crates/order-domain/src/status.rs`
- `server/crates/order-application/Cargo.toml`
- `server/crates/order-application/src/lib.rs`
- `server/crates/order-application/src/dto.rs`
- `server/crates/order-application/src/error.rs`
- `server/crates/order-application/src/module.rs`
- `server/crates/order-application/src/ports.rs`
- `server/crates/order-application/src/use_cases/mod.rs`
- `server/crates/order-application/src/use_cases/place_order_from_cart.rs`
- `server/crates/order-application/src/use_cases/cancel_order_by_customer.rs`
- `server/crates/order-application/src/use_cases/accept_order.rs`
- `server/crates/order-application/src/use_cases/start_preparing_order.rs`
- `server/crates/order-application/src/use_cases/mark_order_ready_for_pickup.rs`
- `server/crates/order-application/src/use_cases/complete_order.rs`
- `server/crates/order-application/src/use_cases/reject_order_by_store.rs`
- `server/crates/order-infrastructure-sqlx/Cargo.toml`
- `server/crates/order-infrastructure-sqlx/src/lib.rs`
- `server/crates/order-infrastructure-sqlx/src/module.rs`
- `server/crates/order-infrastructure-sqlx/src/db_order_status.rs`
- `server/crates/order-infrastructure-sqlx/src/order_repository.rs`
- `server/crates/order-infrastructure-sqlx/src/order_read_repository.rs`
- `server/crates/order-infrastructure-sqlx/src/transaction.rs`
- `server/crates/order-infrastructure-sqlx/tests/repositories.rs`

### New files to create

#### Ordering context

- `server/crates/ordering-domain/Cargo.toml`
- `server/crates/ordering-domain/src/lib.rs`
- `server/crates/ordering-domain/src/customer_id.rs`
- `server/crates/ordering-domain/src/error.rs`
- `server/crates/ordering-domain/src/catalog_item_id.rs`
- `server/crates/ordering-domain/src/order.rs`
- `server/crates/ordering-domain/src/order_id.rs`
- `server/crates/ordering-domain/src/order_item.rs`
- `server/crates/ordering-domain/src/status.rs`
- `server/crates/ordering-domain/tests/architecture.rs`
- `server/crates/ordering-application/Cargo.toml`
- `server/crates/ordering-application/src/lib.rs`
- `server/crates/ordering-application/src/dto.rs`
- `server/crates/ordering-application/src/error.rs`
- `server/crates/ordering-application/src/module.rs`
- `server/crates/ordering-application/src/ports.rs`
- `server/crates/ordering-application/src/use_cases/mod.rs`
- `server/crates/ordering-application/src/use_cases/place_order_from_cart.rs`
- `server/crates/ordering-application/src/use_cases/cancel_order_by_customer.rs`
- `server/crates/ordering-infrastructure-sqlx/Cargo.toml`
- `server/crates/ordering-infrastructure-sqlx/src/lib.rs`
- `server/crates/ordering-infrastructure-sqlx/src/module.rs`
- `server/crates/ordering-infrastructure-sqlx/src/db_order_status.rs`
- `server/crates/ordering-infrastructure-sqlx/src/order_repository.rs`
- `server/crates/ordering-infrastructure-sqlx/src/order_read_repository.rs`
- `server/crates/ordering-infrastructure-sqlx/src/transaction.rs`
- `server/crates/ordering-infrastructure-sqlx/tests/repositories.rs`
- `server/crates/database-infrastructure-sqlx/migrations/202604050402_ordering_commercial_contraction.up.sql`
- `server/crates/database-infrastructure-sqlx/migrations/202604050402_ordering_commercial_contraction.down.sql`

#### Fulfillment context

- `server/crates/fulfillment-domain/src/error.rs`
- `server/crates/fulfillment-domain/src/fulfillment_order.rs`
- `server/crates/fulfillment-domain/src/fulfillment_order_id.rs`
- `server/crates/fulfillment-domain/src/status.rs`
- `server/crates/fulfillment-application/src/error.rs`
- `server/crates/fulfillment-application/src/module.rs`
- `server/crates/fulfillment-application/src/ports.rs`
- `server/crates/fulfillment-application/src/use_cases/mod.rs`
- `server/crates/fulfillment-application/src/use_cases/accept_order.rs`
- `server/crates/fulfillment-application/src/use_cases/start_preparing_order.rs`
- `server/crates/fulfillment-application/src/use_cases/mark_order_ready_for_pickup.rs`
- `server/crates/fulfillment-application/src/use_cases/complete_order.rs`
- `server/crates/fulfillment-application/src/use_cases/reject_order_by_store.rs`
- `server/crates/fulfillment-infrastructure-sqlx/Cargo.toml`
- `server/crates/fulfillment-infrastructure-sqlx/src/lib.rs`
- `server/crates/fulfillment-infrastructure-sqlx/src/module.rs`
- `server/crates/fulfillment-infrastructure-sqlx/src/transaction.rs`
- `server/crates/fulfillment-infrastructure-sqlx/src/workflow_gateway.rs`
- `server/crates/fulfillment-infrastructure-sqlx/src/workflow_order_repository.rs`
- `server/crates/fulfillment-infrastructure-sqlx/src/order_read_repository.rs`
- `server/crates/fulfillment-infrastructure-sqlx/tests/repositories.rs`
- `server/crates/database-infrastructure-sqlx/migrations/202604050401_fulfillment_context.up.sql`
- `server/crates/database-infrastructure-sqlx/migrations/202604050401_fulfillment_context.down.sql`

#### App shell split

- `server/apps/api/src/composition/contexts/ordering.rs`
- `server/apps/api/src/composition/contexts/fulfillment.rs`
- `server/apps/api/src/routes/ordering.rs`
- `server/apps/api/src/routes/fulfillment.rs`
- `server/apps/api/tests/ordering_context_architecture.rs`
- `server/apps/api/tests/fulfillment_context_architecture.rs`

## Task 1: Create the new Ordering crate group by cloning commercial-only responsibilities

**Files:**
- Modify: `server/Cargo.toml`
- Create: `server/crates/ordering-domain/Cargo.toml`
- Create: `server/crates/ordering-domain/src/lib.rs`
- Create: `server/crates/ordering-domain/src/customer_id.rs`
- Create: `server/crates/ordering-domain/src/error.rs`
- Create: `server/crates/ordering-domain/src/catalog_item_id.rs`
- Create: `server/crates/ordering-domain/src/order.rs`
- Create: `server/crates/ordering-domain/src/order_id.rs`
- Create: `server/crates/ordering-domain/src/order_item.rs`
- Create: `server/crates/ordering-domain/src/status.rs`
- Create: `server/crates/ordering-domain/tests/architecture.rs`
- Create: `server/crates/ordering-application/Cargo.toml`
- Create: `server/crates/ordering-application/src/lib.rs`
- Create: `server/crates/ordering-application/src/dto.rs`
- Create: `server/crates/ordering-application/src/error.rs`
- Create: `server/crates/ordering-application/src/module.rs`
- Create: `server/crates/ordering-application/src/ports.rs`
- Create: `server/crates/ordering-application/src/use_cases/mod.rs`
- Create: `server/crates/ordering-application/src/use_cases/place_order_from_cart.rs`
- Create: `server/crates/ordering-application/src/use_cases/cancel_order_by_customer.rs`
- Create: `server/crates/ordering-infrastructure-sqlx/Cargo.toml`
- Create: `server/crates/ordering-infrastructure-sqlx/src/lib.rs`
- Create: `server/crates/ordering-infrastructure-sqlx/src/module.rs`
- Create: `server/crates/ordering-infrastructure-sqlx/src/db_order_status.rs`
- Create: `server/crates/ordering-infrastructure-sqlx/src/order_repository.rs`
- Create: `server/crates/ordering-infrastructure-sqlx/src/order_read_repository.rs`
- Create: `server/crates/ordering-infrastructure-sqlx/src/transaction.rs`
- Create: `server/crates/ordering-infrastructure-sqlx/tests/repositories.rs`
- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050402_ordering_commercial_contraction.up.sql`
- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050402_ordering_commercial_contraction.down.sql`

- [ ] **Step 1: Write a failing workspace/member test for the new Ordering crates**

```rust
use std::{fs, path::Path};

#[test]
fn workspace_members_include_ordering_context_crates() {
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml"))
            .unwrap();

    for member in [
        "crates/ordering-domain",
        "crates/ordering-application",
        "crates/ordering-infrastructure-sqlx",
    ] {
        assert!(manifest.contains(member), "missing workspace member: {member}");
    }
}
```

- [ ] **Step 2: Run the architecture test before the new crates exist**

Run: `cd server && cargo test -p ordering-food-api --test ordering_context_architecture workspace_members_include_ordering_context_crates`
Expected: FAIL because the `ordering-*` crates do not exist in the workspace yet.

- [ ] **Step 3: Create the Ordering crates by copying only commercial-order responsibilities**

Rules:
- Copy `place_order_from_cart` and `cancel_order_by_customer` into `ordering-application`.
- Keep the order aggregate, item snapshot, price snapshot, and customer-facing status in `ordering-domain`.
- Do **not** copy store-side workflow use cases into `ordering-application`.
- Rename the legacy `menu_item_id` concept to a `Catalog`-aligned name such as `catalog_item_id`; no new `Ordering` file may continue the `menu_*` language.
- `Ordering` 只保留商业真相与顾客侧状态；一旦 `Fulfillment` cutover 完成，任何接单/备餐/完成/拒单之类的 store-side workflow 状态都不得继续由 `ordering.*` 写入或宣称所有权。

- [ ] **Step 4: Run targeted tests for the new Ordering crate group**

Run: `cd server && cargo test -p ordering-food-ordering-domain -p ordering-food-ordering-application -p ordering-food-ordering-infrastructure-sqlx --lib`
Expected: PASS for compilation and unit tests in the new crate group.

- [ ] **Step 5: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server/Cargo.toml \
  server/crates/ordering-domain \
  server/crates/ordering-application \
  server/crates/ordering-infrastructure-sqlx \
  server/apps/api/tests/ordering_context_architecture.rs
git commit -m "refactor: introduce ordering context crate group"
```

## Task 2: Create the Fulfillment crate group around store-side workflow progression

**Files:**
- Modify: `server/Cargo.toml`
- Modify: `server/crates/database-infrastructure-sqlx/src/bin/migration-info.rs`
- Modify: `server/crates/fulfillment-domain/Cargo.toml`
- Modify: `server/crates/fulfillment-domain/src/lib.rs`
- Create: `server/crates/fulfillment-domain/src/error.rs`
- Create: `server/crates/fulfillment-domain/src/fulfillment_order.rs`
- Create: `server/crates/fulfillment-domain/src/fulfillment_order_id.rs`
- Create: `server/crates/fulfillment-domain/src/status.rs`
- Modify: `server/crates/fulfillment-application/Cargo.toml`
- Modify: `server/crates/fulfillment-application/src/lib.rs`
- Create: `server/crates/fulfillment-application/src/error.rs`
- Create: `server/crates/fulfillment-application/src/module.rs`
- Create: `server/crates/fulfillment-application/src/ports.rs`
- Create: `server/crates/fulfillment-application/src/use_cases/mod.rs`
- Create: `server/crates/fulfillment-application/src/use_cases/accept_order.rs`
- Create: `server/crates/fulfillment-application/src/use_cases/start_preparing_order.rs`
- Create: `server/crates/fulfillment-application/src/use_cases/mark_order_ready_for_pickup.rs`
- Create: `server/crates/fulfillment-application/src/use_cases/complete_order.rs`
- Create: `server/crates/fulfillment-application/src/use_cases/reject_order_by_store.rs`
- Create: `server/crates/fulfillment-infrastructure-sqlx/Cargo.toml`
- Create: `server/crates/fulfillment-infrastructure-sqlx/src/lib.rs`
- Create: `server/crates/fulfillment-infrastructure-sqlx/src/module.rs`
- Create: `server/crates/fulfillment-infrastructure-sqlx/src/transaction.rs`
- Create: `server/crates/fulfillment-infrastructure-sqlx/src/workflow_gateway.rs`
- Create: `server/crates/fulfillment-infrastructure-sqlx/src/workflow_order_repository.rs`
- Create: `server/crates/fulfillment-infrastructure-sqlx/src/order_read_repository.rs`
- Create: `server/crates/fulfillment-infrastructure-sqlx/tests/repositories.rs`
- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050401_fulfillment_context.up.sql`
- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050401_fulfillment_context.down.sql`
- Create: `server/apps/api/tests/fulfillment_context_architecture.rs`

- [ ] **Step 1: Write a failing architecture test that forbids customer-order use cases from living in Fulfillment**

```rust
use std::{fs, path::Path};

#[test]
fn fulfillment_application_stays_store_workflow_only() {
    let source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/fulfillment-application/src/use_cases/mod.rs"),
    )
    .unwrap();

    assert!(!source.contains("place_order_from_cart"));
    assert!(!source.contains("cancel_order_by_customer"));
}
```

- [ ] **Step 2: Run the test before the fulfillment use cases exist**

Run: `cd server && cargo test -p ordering-food-api --test fulfillment_context_architecture fulfillment_application_stays_store_workflow_only`
Expected: FAIL because the fulfillment use case module does not exist yet.

- [ ] **Step 3: Create the Fulfillment context with store-side workflow use cases only**

Rules:
- `accept_order`
- `start_preparing_order`
- `mark_order_ready_for_pickup`
- `complete_order`
- `reject_order_by_store`

All five use cases move into `fulfillment-application`.

- [ ] **Step 4: Create the `fulfillment` schema, backfill existing workflow state, and declare the only temporary sync bridges**

Rules:
- Add a new migration pair `202604050401_fulfillment_context.*` that creates `fulfillment`-owned tables for store-side workflow truth, at minimum one write model table such as `fulfillment.workflow_orders`.
- Backfill existing store-side state from `ordering.orders` into `fulfillment.workflow_orders` so cutover does not depend on future Phase 3 events.
- After this cutover, only `Fulfillment` may mutate store-side workflow state; `Ordering` must retain only commercial/customer-facing truth.
- Add a paired contraction migration `202604050402_ordering_commercial_contraction.*` that removes or retires store-side workflow status ownership from `ordering.*`, and update `migration-info.rs` in the same task.
- Define two temporary synchronous published contracts and no more:
  1. `Ordering -> Fulfillment` bootstrap for newly placed or customer-cancelled orders
  2. `Fulfillment -> Ordering` commercial-state guard for workflow transitions that need immediate validation
- Consume these bridges through ports and implement them as explicit temporary whitelist exceptions in infrastructure/app-shell wiring.
- Document in code comments and tests that both bridges are Phase 2 debt scheduled for removal in Phase 3.

- [ ] **Step 5: Run targeted tests for the Fulfillment crate group**

Run: `cd server && cargo test -p ordering-food-fulfillment-domain -p ordering-food-fulfillment-application -p ordering-food-fulfillment-infrastructure-sqlx --lib && cargo test -p ordering-food-database-infrastructure-sqlx fulfillment_context --lib && cargo test -p ordering-food-database-infrastructure-sqlx --bin migration-info`
Expected: PASS for compilation, unit tests, the new `fulfillment` / `ordering` contraction migrations, and updated migration ordering checks.

- [ ] **Step 6: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server/Cargo.toml \
  server/crates/database-infrastructure-sqlx/src/bin/migration-info.rs \
  server/crates/fulfillment-domain \
  server/crates/fulfillment-application \
  server/crates/fulfillment-infrastructure-sqlx \
  server/crates/database-infrastructure-sqlx/migrations/202604050401_fulfillment_context.up.sql \
  server/crates/database-infrastructure-sqlx/migrations/202604050401_fulfillment_context.down.sql \
  server/crates/database-infrastructure-sqlx/migrations/202604050402_ordering_commercial_contraction.up.sql \
  server/crates/database-infrastructure-sqlx/migrations/202604050402_ordering_commercial_contraction.down.sql \
  server/crates/ordering-published/src/lib.rs \
  server/apps/api/tests/fulfillment_context_architecture.rs
git commit -m "refactor: introduce fulfillment context crate group"
```

## Task 3: Split the app shell into Ordering routes/context and Fulfillment routes/context

**Files:**
- Modify: `server/apps/api/Cargo.toml`
- Modify: `server/apps/api/src/composition/contexts/mod.rs`
- Create: `server/apps/api/src/composition/contexts/ordering.rs`
- Create: `server/apps/api/src/composition/contexts/fulfillment.rs`
- Modify: `server/apps/api/src/routes/mod.rs`
- Create: `server/apps/api/src/routes/ordering.rs`
- Create: `server/apps/api/src/routes/fulfillment.rs`
- Modify: `server/apps/api/src/ts_bindings.rs`
- Modify: `server/apps/api/src/composition/context_registration.rs`

- [ ] **Step 1: Write a failing route-shell architecture test for the split**

```rust
use std::{fs, path::Path};

#[test]
fn route_modules_separate_ordering_and_fulfillment_http_contracts() {
    let mod_source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/routes/mod.rs"))
            .unwrap();

    assert!(mod_source.contains("pub mod ordering;"));
    assert!(mod_source.contains("pub mod fulfillment;"));
    assert!(!mod_source.contains("pub mod orders;"));
}
```

- [ ] **Step 2: Run the route architecture test before the split**

Run: `cd server && cargo test -p ordering-food-api --test ordering_context_architecture route_modules_separate_ordering_and_fulfillment_http_contracts`
Expected: FAIL because the app shell still exports `orders`.

- [ ] **Step 3: Split the HTTP layer without changing external behavior unless explicitly necessary**

Rules:
- `ordering.rs` owns customer-facing endpoints:
  - list orders
  - get order
  - place order
  - cancel by customer
- `fulfillment.rs` owns store-facing workflow endpoints:
  - accept
  - start preparing
  - ready
  - complete
  - reject
- 如果现有 HTTP 合约仍需要组合展示商业状态与履约状态，只允许在 app shell 做临时组合读，不允许把两个上下文的状态重新并回同一个领域模型。
- Existing URL paths may stay stable in this phase if that avoids unnecessary frontend churn.

- [ ] **Step 4: Split composition wiring into `register_ordering` and `register_fulfillment`**

Rules:
- `apps/api` may depend on both `ordering-application` and `fulfillment-application`.
- `ordering` 与 `fulfillment` wiring 若仍需临时同步协作，只能通过前面声明的两个 explicit whitelist seams。
- Do not let `apps/api` reintroduce old `order-*` coupling after the split.

- [ ] **Step 5: Run the app-shell test suite for the route/context split**

Run: `cd server && cargo test -p ordering-food-api --test ordering_context_architecture --test fulfillment_context_architecture && cargo test -p ordering-food-api routes::ordering --lib && cargo test -p ordering-food-api routes::fulfillment --lib`
Expected: PASS for the new architecture tests and the split route modules.

- [ ] **Step 6: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server/apps/api/Cargo.toml \
  server/apps/api/src/composition/contexts/mod.rs \
  server/apps/api/src/composition/contexts/ordering.rs \
  server/apps/api/src/composition/contexts/fulfillment.rs \
  server/apps/api/src/routes/mod.rs \
  server/apps/api/src/routes/ordering.rs \
  server/apps/api/src/routes/fulfillment.rs \
  server/apps/api/src/ts_bindings.rs \
  server/apps/api/tests/ordering_context_architecture.rs \
  server/apps/api/tests/fulfillment_context_architecture.rs
git commit -m "refactor: split order app shell into ordering and fulfillment"
```

## Task 4: Cut infrastructure and tests over from legacy `order-*` to `ordering-*` / `fulfillment-*`

**Files:**
- Modify: `server/crates/ordering-infrastructure-sqlx/tests/repositories.rs`
- Modify: `server/crates/fulfillment-infrastructure-sqlx/tests/repositories.rs`
- Modify: `server/apps/api/src/routes/ordering.rs`
- Modify: `server/apps/api/src/routes/fulfillment.rs`
- Delete: `server/apps/api/src/routes/orders.rs`
- Delete: `server/apps/api/src/composition/contexts/order.rs`
- Delete: `server/crates/order-domain/**`
- Delete: `server/crates/order-application/**`
- Delete: `server/crates/order-infrastructure-sqlx/**`
- Modify: `server/Cargo.toml`

- [ ] **Step 1: Write a failing workspace guard that forbids legacy `order-*` members after cutover**

```rust
use std::{fs, path::Path};

#[test]
fn workspace_members_no_longer_include_legacy_order_crates() {
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml"))
            .unwrap();

    for member in [
        "crates/order-domain",
        "crates/order-application",
        "crates/order-infrastructure-sqlx",
    ] {
        assert!(!manifest.contains(member), "legacy member still present: {member}");
    }
}
```

- [ ] **Step 2: Run the workspace guard before deleting the legacy crates**

Run: `cd server && cargo test -p ordering-food-api --test ordering_context_architecture workspace_members_no_longer_include_legacy_order_crates`
Expected: FAIL because the old `order-*` members are still present.

- [ ] **Step 3: Move remaining tests and integration wiring to the new context names**

Rules:
- Repository tests that validate order commercial persistence move to `ordering-infrastructure-sqlx`.
- Store workflow gateway and staff-flow tests move to `fulfillment-infrastructure-sqlx`.
- `apps/api` route tests must be split the same way as the route modules.

- [ ] **Step 4: Delete the legacy `order-*` crate group and old app-shell files**

Rules:
- Delete only after the new crates compile and tests pass.
- Do not leave mixed `order-*` and `ordering-*` module registration in the same final state.

- [ ] **Step 5: Run a post-cutover verification pass**

Run: `cd server && cargo test -p ordering-food-ordering-domain -p ordering-food-ordering-application -p ordering-food-ordering-infrastructure-sqlx -p ordering-food-fulfillment-domain -p ordering-food-fulfillment-application -p ordering-food-fulfillment-infrastructure-sqlx && cargo test -p ordering-food-api`
Expected: PASS for the new context crates and the app-shell suite.

- [ ] **Step 6: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server/Cargo.toml \
  server/apps/api \
  server/crates/ordering-domain \
  server/crates/ordering-application \
  server/crates/ordering-infrastructure-sqlx \
  server/crates/fulfillment-domain \
  server/crates/fulfillment-application \
  server/crates/fulfillment-infrastructure-sqlx
git rm -r server/crates/order-domain \
  server/crates/order-application \
  server/crates/order-infrastructure-sqlx \
  server/apps/api/src/routes/orders.rs \
  server/apps/api/src/composition/contexts/order.rs
git commit -m "refactor: replace legacy order context with ordering and fulfillment"
```

## Task 5: Record and enforce the temporary synchronous exception so Phase 3 can remove it cleanly

**Files:**
- Modify: `server/crates/ordering-published/src/lib.rs`
- Create: `server/apps/api/tests/ordering_sync_exception_architecture.rs`
- Modify: `docs/superpowers/specs/2026-04-05-backend-ddd-target-architecture-design.md`

- [ ] **Step 1: Write a failing guard that documents the only allowed temporary sync paths**

```rust
use std::{fs, path::Path};

#[test]
fn only_whitelisted_sync_collaboration_between_ordering_and_fulfillment_exists() {
    let source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/ordering-published/src/lib.rs"),
    )
    .unwrap();

    assert!(source.contains("TemporarySyncFulfillmentWorkflowGateway"));
    assert!(source.contains("TemporarySyncFulfillmentBootstrapGateway"));
}
```

- [ ] **Step 2: Run the test before the whitelist contract exists**

Run: `cd server && cargo test -p ordering-food-api --test ordering_sync_exception_architecture`
Expected: FAIL because the explicit temporary sync whitelist contracts have not been declared.

- [ ] **Step 3: Declare the temporary published contract and document removal criteria**

Rules:
- The contract names must carry `TemporarySync` or equivalent explicit debt labeling.
- The spec/doc note must reference Phase 3 as the removal point for both bridges.
- The implementation plan comment must explain why these exceptions exist and why they are not the long-term collaboration model.
- No third temporary sync path may be added without re-opening the architecture plan.

- [ ] **Step 4: Run the whitelist guard**

Run: `cd server && cargo test -p ordering-food-api --test ordering_sync_exception_architecture`
Expected: PASS and both temporary bridges are now explicit and searchable.

- [ ] **Step 5: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server/crates/ordering-published/src/lib.rs \
  server/apps/api/tests/ordering_sync_exception_architecture.rs \
  docs/superpowers/specs/2026-04-05-backend-ddd-target-architecture-design.md
git commit -m "docs: record temporary ordering fulfillment sync exception"
```

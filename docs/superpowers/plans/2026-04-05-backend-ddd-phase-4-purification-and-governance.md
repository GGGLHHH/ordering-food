# Backend DDD Phase 4 Purification And Governance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Finish the backend DDD purification by removing temporary cross-context shortcuts, enforcing schema ownership and sync-exception governance, and adding the long-term safety rails required to keep the architecture pure after migration.

**Architecture:** By this phase, the six target contexts and the event spine already exist. The remaining work is to turn the architecture from “currently arranged this way” into “mechanically forced to stay this way”. That means explicit sync-exception whitelists, hard architecture guards, projection rebuild tooling, idempotent projector behavior, and published contract evolution rules that keep future changes from silently eroding the model.

**Tech Stack:** Rust 2024, Cargo workspace, Tokio, SQLx, tracing, optional OpenTelemetry, testcontainers, wiremock, architecture tests based on manifest/source scanning and SQL inspection, projection rebuild CLI tooling.

---

## Scope Check

This plan assumes the following have already landed:

1. `Phase 1 boundary skeleton`
2. `Phase 2A Organization foundation`
3. `Phase 2B Access + Identity purification`
4. `Phase 2C Catalog migration`
5. `Phase 2D Ordering / Fulfillment split`
6. `Phase 3 event spine and projections`

This plan does **not** introduce new business capabilities. It only:

1. Removes temporary architectural debt
2. Hardens collaboration rules
3. Adds operational and testing guardrails

## Planned File Map

### Existing files to modify

- `server/Cargo.toml`
  Add any final tooling crates required for projection rebuild and contract governance.
- `server/apps/api/Cargo.toml`
  Add optional observability / contract test tooling needed by the final governance stage.
- `server/apps/api/src/app.rs`
  Finalize runtime startup / shutdown responsibilities for projector rebuild and sync whitelist enforcement.
- `server/apps/api/src/observability.rs`
  Extend tracing fields to include correlation / causation propagation for async chains.
- `server/apps/api/src/composition/platform.rs`
  Remove any temporary sync bridge providers that survived earlier phases.
- `server/crates/platform-kernel/src/lib.rs`
  Extend neutral identifiers with any missing `MessageId` / `CausationId` / `EventId` contracts if Phase 3 left them partial.
- `server/crates/ordering-published/src/lib.rs`
  Remove any temporary sync bridge contracts that should not survive final purification.
- `server/crates/fulfillment-integration/src/lib.rs`
  Drop any temporary bridge re-exports that were only used during Phase 2 migration.
- `server/crates/database-infrastructure-sqlx/src/lib.rs`
  Export rebuild helpers, migrator access, and any SQL ownership inspection helpers.

### New files to create

#### Governance and architecture guards

- `server/apps/api/tests/sync_exception_architecture.rs`
- `server/apps/api/tests/schema_ownership_architecture.rs`
- `server/apps/api/tests/published_contract_architecture.rs`
- `server/apps/api/tests/projection_idempotency_architecture.rs`
- `server/apps/api/tests/no_cross_context_sql_architecture.rs`
- `server/apps/api/tests/no_internal_context_imports_architecture.rs`

#### Projection rebuild and operational tooling

- `server/crates/database-infrastructure-sqlx/src/bin/rebuild-projections.rs`
- `server/crates/database-infrastructure-sqlx/src/rebuild.rs`
- `server/crates/database-infrastructure-sqlx/tests/rebuild.rs`

#### Contract evolution and compatibility tests

- `server/crates/identity-published/tests/contracts.rs`
- `server/crates/organization-published/tests/contracts.rs`
- `server/crates/catalog-published/tests/contracts.rs`
- `server/crates/access-published/tests/contracts.rs`
- `server/crates/ordering-published/tests/contracts.rs`
- `server/crates/fulfillment-published/tests/contracts.rs`

#### Async observability and runtime support

- `server/apps/api/tests/observability_async_runtime.rs`
- `server/crates/platform-messaging/tests/idempotency.rs`
- `server/crates/platform-events/tests/versioning.rs`

## Task 1: Replace informal sync exceptions with a hard whitelist registry

**Files:**
- Modify: `server/apps/api/src/composition/platform.rs`
- Modify: `server/crates/ordering-published/src/lib.rs`
- Create: `server/apps/api/tests/sync_exception_architecture.rs`

- [ ] **Step 1: Write a failing architecture test that every remaining cross-context sync call is explicitly whitelisted**

```rust
use std::{fs, path::Path};

#[test]
fn sync_exception_registry_is_explicit_and_searchable() {
    let source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/composition/platform.rs"),
    )
    .unwrap();

    assert!(source.contains("SYNC_EXCEPTION_WHITELIST"));
}
```

- [ ] **Step 2: Run the test before the registry exists**

Run: `cd server && cargo test -p ordering-food-api --test sync_exception_architecture sync_exception_registry_is_explicit_and_searchable`
Expected: FAIL because the sync whitelist registry has not been formalized yet.

- [ ] **Step 3: Add the sync exception registry and delete unapproved bridges**

Rules:
- Every remaining sync exception must be listed in one place.
- Each whitelist item must record reason and removal conditions.
- Any leftover bridge that is not justified must be deleted in this phase.
- The long-term target remains “cross-context default = events + projections”.

- [ ] **Step 4: Run the sync exception architecture tests**

Run: `cd server && cargo test -p ordering-food-api --test sync_exception_architecture`
Expected: PASS and the sync surface is now explicit.

- [ ] **Step 5: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server/apps/api/src/composition/platform.rs \
  server/crates/ordering-published/src/lib.rs \
  server/apps/api/tests/sync_exception_architecture.rs
git commit -m "refactor: enforce sync exception whitelist"
```

## Task 2: Enforce schema ownership and ban cross-context SQL reach-through

**Files:**
- Create: `server/apps/api/tests/schema_ownership_architecture.rs`
- Create: `server/apps/api/tests/no_cross_context_sql_architecture.rs`
- Modify: `server/crates/database-infrastructure-sqlx/src/lib.rs`

- [ ] **Step 1: Write a failing architecture test that forbids direct SQL references to foreign schemas**

```rust
use std::{fs, path::Path};

#[test]
fn persistence_code_does_not_query_foreign_context_schemas() {
    let source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/ordering-infrastructure-sqlx/src/order_read_repository.rs"),
    )
    .unwrap();

    assert!(!source.contains("JOIN access."));
    assert!(!source.contains("JOIN catalog."));
    assert!(!source.contains("JOIN organization."));
}
```

- [ ] **Step 2: Run the test before the final ownership sweep**

Run: `cd server && cargo test -p ordering-food-api --test no_cross_context_sql_architecture`
Expected: FAIL if any remaining foreign-schema reach-through still exists.

- [ ] **Step 3: Add final ownership guards and clean remaining violations**

Rules:
- Persistence crates may only query their own schema plus owned projection tables.
- Projection tables must sit under the owning context schema.
- No cross-context SQL join may remain in business logic.

- [ ] **Step 4: Run the schema ownership guard suite**

Run: `cd server && cargo test -p ordering-food-api --test schema_ownership_architecture --test no_cross_context_sql_architecture`
Expected: PASS and schema ownership is enforceable.

- [ ] **Step 5: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server/apps/api/tests/schema_ownership_architecture.rs \
  server/apps/api/tests/no_cross_context_sql_architecture.rs \
  server/crates/database-infrastructure-sqlx/src/lib.rs
git commit -m "test: enforce schema ownership boundaries"
```

## Task 3: Add projection rebuild tooling and projector idempotency verification

**Files:**
- Create: `server/crates/database-infrastructure-sqlx/src/bin/rebuild-projections.rs`
- Create: `server/crates/database-infrastructure-sqlx/src/rebuild.rs`
- Create: `server/crates/database-infrastructure-sqlx/tests/rebuild.rs`
- Create: `server/apps/api/tests/projection_idempotency_architecture.rs`
- Create: `server/crates/platform-messaging/tests/idempotency.rs`

- [ ] **Step 1: Write a failing test for projection rebuild support**

```rust
#[test]
fn rebuild_binary_is_present() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/database-infrastructure-sqlx/src/bin/rebuild-projections.rs");
    assert!(path.exists());
}
```

- [ ] **Step 2: Run the test before the rebuild tool exists**

Run: `cd server && cargo test -p ordering-food-api --test projection_idempotency_architecture rebuild_binary_is_present`
Expected: FAIL because the rebuild tool has not been created yet.

- [ ] **Step 3: Implement projection rebuild entrypoints and idempotency tests**

Rules:
- Rebuild must be able to replay from outbox or event store source-of-truth for first-wave projections.
- Projectors must tolerate duplicate delivery.
- Checkpoints must support resume after partial failure.

- [ ] **Step 4: Run rebuild and idempotency tests**

Run: `cd server && cargo test -p ordering-food-database-infrastructure-sqlx rebuild --lib && cargo test -p ordering-food-platform-messaging --test idempotency`
Expected: PASS for rebuild helpers and projector idempotency.

- [ ] **Step 5: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server/crates/database-infrastructure-sqlx/src/bin/rebuild-projections.rs \
  server/crates/database-infrastructure-sqlx/src/rebuild.rs \
  server/crates/database-infrastructure-sqlx/tests/rebuild.rs \
  server/apps/api/tests/projection_idempotency_architecture.rs \
  server/crates/platform-messaging/tests/idempotency.rs
git commit -m "feat: add projection rebuild and idempotency verification"
```

## Task 4: Lock down published contract evolution and ban internal imports across contexts

**Files:**
- Create: `server/apps/api/tests/published_contract_architecture.rs`
- Create: `server/apps/api/tests/no_internal_context_imports_architecture.rs`
- Create: `server/crates/identity-published/tests/contracts.rs`
- Create: `server/crates/organization-published/tests/contracts.rs`
- Create: `server/crates/catalog-published/tests/contracts.rs`
- Create: `server/crates/access-published/tests/contracts.rs`
- Create: `server/crates/ordering-published/tests/contracts.rs`
- Create: `server/crates/fulfillment-published/tests/contracts.rs`

- [ ] **Step 1: Write a failing guard that application crates must not import foreign internal layers**

```rust
use std::{fs, path::Path};

#[test]
fn ordering_application_does_not_import_catalog_application_or_domain() {
    let source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/ordering-application/src/ports.rs"),
    )
    .unwrap();

    assert!(!source.contains("ordering_food_catalog_application"));
    assert!(!source.contains("ordering_food_catalog_domain"));
}
```

- [ ] **Step 2: Run the guard before the final import sweep**

Run: `cd server && cargo test -p ordering-food-api --test no_internal_context_imports_architecture`
Expected: FAIL if any internal foreign-layer imports still exist.

- [ ] **Step 3: Add published contract compatibility tests**

Rules:
- Published crates test serialization shape and required fields.
- Breaking changes must fail contract tests.
- Application crates may import only foreign `*-published`.

- [ ] **Step 4: Run architecture and contract suites**

Run: `cd server && cargo test -p ordering-food-api --test published_contract_architecture --test no_internal_context_imports_architecture && cargo test -p ordering-food-identity-published -p ordering-food-organization-published -p ordering-food-catalog-published -p ordering-food-access-published -p ordering-food-ordering-published -p ordering-food-fulfillment-published`
Expected: PASS for architecture guards and published contract tests.

- [ ] **Step 5: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server/apps/api/tests/published_contract_architecture.rs \
  server/apps/api/tests/no_internal_context_imports_architecture.rs \
  server/crates/identity-published/tests/contracts.rs \
  server/crates/organization-published/tests/contracts.rs \
  server/crates/catalog-published/tests/contracts.rs \
  server/crates/access-published/tests/contracts.rs \
  server/crates/ordering-published/tests/contracts.rs \
  server/crates/fulfillment-published/tests/contracts.rs
git commit -m "test: lock published contract and import boundaries"
```

## Task 5: Finish async observability and final workspace verification

**Files:**
- Modify: `server/apps/api/Cargo.toml`
- Modify: `server/apps/api/src/observability.rs`
- Modify: `server/apps/api/src/app.rs`
- Modify: `server/crates/platform-kernel/src/lib.rs`
- Create: `server/apps/api/tests/observability_async_runtime.rs`
- Create: `server/crates/platform-events/tests/versioning.rs`

- [ ] **Step 1: Write a failing test for correlation / causation propagation support**

```rust
use std::{fs, path::Path};

#[test]
fn platform_kernel_exposes_causation_and_correlation_ids() {
    let source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/platform-kernel/src/lib.rs"),
    )
    .unwrap();

    assert!(source.contains("CausationId"));
    assert!(source.contains("CorrelationId"));
}
```

- [ ] **Step 2: Run the test before the final kernel extension**

Run: `cd server && cargo test -p ordering-food-api --test observability_async_runtime platform_kernel_exposes_causation_and_correlation_ids`
Expected: FAIL if the final async-trace identifiers are still incomplete.

- [ ] **Step 3: Extend observability and versioning safety**

Rules:
- Correlation / causation metadata must propagate through dispatcher and projector logs.
- Published event versioning tests must catch incompatible schema drift.
- App startup / shutdown must keep async runtime cleanup deterministic.

- [ ] **Step 4: Run final verification commands**

Run: `cd server && cargo test --workspace --exclude ordering-food-authz-infrastructure-sqlx --exclude ordering-food-menu-infrastructure-sqlx --exclude ordering-food-order-infrastructure-sqlx --exclude ordering-food-identity-infrastructure-sqlx --exclude ordering-food-database-infrastructure-sqlx`
Expected: PASS for the non-database workspace verification.

Run: `cd server && DATABASE_URL=postgres://... cargo test -p ordering-food-database-infrastructure-sqlx`
Expected: PASS for database-backed rebuild and migration verification.

- [ ] **Step 5: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server/apps/api/Cargo.toml \
  server/apps/api/src/observability.rs \
  server/apps/api/src/app.rs \
  server/crates/platform-kernel/src/lib.rs \
  server/apps/api/tests/observability_async_runtime.rs \
  server/crates/platform-events/tests/versioning.rs
git commit -m "refactor: finalize async observability and governance checks"
```

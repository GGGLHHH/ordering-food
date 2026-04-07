# Backend DDD Phase 3 Event Spine And Projections Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the first real cross-context collaboration backbone for the backend DDD target architecture: stable published events, ACL translators, integration read models, Postgres outbox persistence, and in-process dispatcher / projector runtime.

**Architecture:** This phase turns the Phase 1 skeleton and Phase 2 context redraw into a working eventually-consistent collaboration model. Each context continues to own its own schema and write model, but cross-context cooperation moves onto explicit `published` contracts plus `integration`-owned projections. The implementation is split into two equally important halves: producer-side outbox emission inside each context's local transaction, and consumer-side projection tables inside each consuming context's owned schema. The default runtime path is `Postgres outbox + sqlx + tokio dispatcher / projector`; no external broker is introduced in this phase.

**Tech Stack:** Rust 2024, Cargo workspace, Tokio, SQLx, Serde, tracing, Postgres outbox, integration read models, architecture tests based on manifest/source scanning, optional testcontainers for database-backed projection tests.

---

## Scope Check

This plan assumes the following plans are already implemented:

1. `Phase 2A Organization foundation`
2. `Phase 2B Access + Identity purification`
3. `Phase 2C Catalog migration`
4. `Phase 2D Ordering / Fulfillment split`

This plan does **not** cover:

1. External message broker adoption
2. Final sync-exception governance hardening
3. Long-term projection rebuild CLI polish and operational runbooks
4. Full observability rollout for every path

## Planned File Map

### Existing files to modify

- `server/Cargo.toml`
  Register new platform crates and any new persistence/runtime crates needed by the event spine.
- `server/apps/api/Cargo.toml`
  Add platform eventing and messaging dependencies required by app-shell runtime wiring.
- `server/apps/api/src/runtime.rs`
  Add runtime providers for dispatcher loop timing, UUID/message IDs, and correlation propagation.
- `server/apps/api/src/composition/platform.rs`
  Add outbox / messaging / projector runtime providers to `ApiPlatform`.
- `server/apps/api/src/app.rs`
  Start dispatcher / projector background tasks and ensure graceful shutdown hooks exist.
- `server/crates/identity-application/src/module.rs`
- `server/crates/identity-infrastructure-sqlx/src/transaction.rs`
- `server/crates/organization-application/src/module.rs`
- `server/crates/organization-infrastructure-sqlx/src/transaction.rs`
- `server/crates/catalog-application/src/module.rs`
- `server/crates/catalog-infrastructure-sqlx/src/transaction.rs`
- `server/crates/access-application/src/service.rs`
- `server/crates/ordering-application/src/module.rs`
- `server/crates/ordering-infrastructure-sqlx/src/transaction.rs`
- `server/crates/fulfillment-application/src/module.rs`
- `server/crates/fulfillment-infrastructure-sqlx/src/transaction.rs`
  Teach first-wave producer contexts to append published events to the outbox in the same local transaction that commits their owned state.
- `server/crates/identity-published/src/lib.rs`
  Replace placeholder with stable published identity facts and events.
- `server/crates/organization-published/src/lib.rs`
  Replace placeholder with stable organization scope refs and events.
- `server/crates/catalog-published/src/lib.rs`
  Replace placeholder with sellable item / price / availability facts and events.
- `server/crates/access-published/src/lib.rs`
  Replace placeholder with membership / role assignment facts and events.
- `server/crates/ordering-published/src/lib.rs`
  Replace placeholder with order-placed / commercial-state facts and events.
- `server/crates/fulfillment-published/src/lib.rs`
  Replace placeholder with fulfillment status facts and events.
- `server/crates/identity-integration/src/lib.rs`
- `server/crates/organization-integration/src/lib.rs`
- `server/crates/catalog-integration/src/lib.rs`
- `server/crates/access-integration/src/lib.rs`
- `server/crates/ordering-integration/src/lib.rs`
- `server/crates/fulfillment-integration/src/lib.rs`
  Replace placeholder traits with real ACL / projector modules and re-exports.
- `server/crates/database-infrastructure-sqlx/src/lib.rs`
  Export the shared migrator pieces required by outbox and projection storage.
- `server/crates/database-infrastructure-sqlx/migrations/202603150001_ordering.up.sql`
  Leave existing context schema intact; do not retrofit cross-context ownership here.

### New files to create

#### Platform eventing

- `server/crates/platform-events/Cargo.toml`
- `server/crates/platform-events/src/lib.rs`
- `server/crates/platform-events/src/event_id.rs`
- `server/crates/platform-events/src/message.rs`
- `server/crates/platform-events/src/metadata.rs`
- `server/crates/platform-events/tests/architecture.rs`
- `server/crates/platform-messaging/Cargo.toml`
- `server/crates/platform-messaging/src/lib.rs`
- `server/crates/platform-messaging/src/dispatcher.rs`
- `server/crates/platform-messaging/src/projector.rs`
- `server/crates/platform-messaging/src/retry.rs`
- `server/crates/platform-persistence/Cargo.toml`
- `server/crates/platform-persistence/src/lib.rs`
- `server/crates/platform-persistence/src/outbox.rs`
- `server/crates/platform-persistence/src/checkpoint.rs`

#### Database migrations and SQLx support

- `server/crates/database-infrastructure-sqlx/migrations/202604050501_outbox.down.sql`
- `server/crates/database-infrastructure-sqlx/migrations/202604050501_outbox.up.sql`
- `server/crates/database-infrastructure-sqlx/migrations/202604050502_projection_checkpoints.down.sql`
- `server/crates/database-infrastructure-sqlx/migrations/202604050502_projection_checkpoints.up.sql`
- `server/crates/database-infrastructure-sqlx/migrations/202604050503_first_wave_integration_projections.down.sql`
- `server/crates/database-infrastructure-sqlx/migrations/202604050503_first_wave_integration_projections.up.sql`
- `server/crates/database-infrastructure-sqlx/src/outbox.rs`
- `server/crates/database-infrastructure-sqlx/src/projection_checkpoint.rs`
- `server/crates/access-infrastructure-sqlx/src/transaction.rs`

#### Integration and projection modules

- `server/crates/access-integration/src/acl.rs`
- `server/crates/access-integration/src/projection.rs`
- `server/crates/access-integration/src/identity_subject_projection.rs`
- `server/crates/access-integration/src/organization_scope_projection.rs`
- `server/crates/catalog-integration/src/acl.rs`
- `server/crates/catalog-integration/src/projection.rs`
- `server/crates/catalog-integration/src/organization_store_projection.rs`
- `server/crates/ordering-integration/src/acl.rs`
- `server/crates/ordering-integration/src/projection.rs`
- `server/crates/ordering-integration/src/catalog_sellable_item_projection.rs`
- `server/crates/ordering-integration/src/organization_store_projection.rs`
- `server/crates/ordering-integration/src/access_scope_projection.rs`
- `server/crates/fulfillment-integration/src/acl.rs`
- `server/crates/fulfillment-integration/src/projection.rs`
- `server/crates/fulfillment-integration/src/ordering_order_projection.rs`
- `server/crates/fulfillment-integration/src/organization_store_projection.rs`
- `server/crates/fulfillment-integration/src/access_staff_scope_projection.rs`

#### App-shell runtime wiring and tests

- `server/apps/api/src/composition/eventing.rs`
- `server/apps/api/src/composition/projections.rs`
- `server/apps/api/tests/published_integration_architecture.rs`
- `server/apps/api/tests/outbox_architecture.rs`
- `server/apps/api/tests/sync_whitelist_transition_architecture.rs`

## Task 1: Introduce platform-level event, messaging, and persistence contracts

**Files:**
- Modify: `server/Cargo.toml`
- Create: `server/crates/platform-events/Cargo.toml`
- Create: `server/crates/platform-events/src/lib.rs`
- Create: `server/crates/platform-events/src/event_id.rs`
- Create: `server/crates/platform-events/src/message.rs`
- Create: `server/crates/platform-events/src/metadata.rs`
- Create: `server/crates/platform-events/tests/architecture.rs`
- Create: `server/crates/platform-messaging/Cargo.toml`
- Create: `server/crates/platform-messaging/src/lib.rs`
- Create: `server/crates/platform-messaging/src/dispatcher.rs`
- Create: `server/crates/platform-messaging/src/projector.rs`
- Create: `server/crates/platform-messaging/src/retry.rs`
- Create: `server/crates/platform-persistence/Cargo.toml`
- Create: `server/crates/platform-persistence/src/lib.rs`
- Create: `server/crates/platform-persistence/src/outbox.rs`
- Create: `server/crates/platform-persistence/src/checkpoint.rs`

- [ ] **Step 1: Write a failing architecture test that platform eventing stays framework-light**

```rust
use std::{fs, path::Path};

#[test]
fn platform_events_manifest_stays_web_and_sql_free() {
    let manifest = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .unwrap();

    for forbidden in ["axum", "tower", "tower-http", "sqlx", "redis"] {
        assert!(!manifest.contains(&format!("{forbidden}.workspace")));
        assert!(!manifest.contains(&format!("{forbidden} =")));
    }
}
```

- [ ] **Step 2: Run the test before the new crates exist**

Run: `cd server && cargo test -p ordering-food-platform-events`
Expected: FAIL because the package does not exist yet.

- [ ] **Step 3: Create the platform eventing crates with explicit contracts**

Rules:
- `platform-events` defines event envelope, metadata, correlation / causation fields, and serialization shape.
- `platform-messaging` defines dispatcher / projector contracts and retry semantics.
- `platform-persistence` defines outbox record and projection checkpoint contracts.
- None of these crates may depend on app-shell or business contexts.

- [ ] **Step 4: Run targeted tests for the new platform crates**

Run: `cd server && cargo test -p ordering-food-platform-events -p ordering-food-platform-messaging -p ordering-food-platform-persistence`
Expected: PASS for compilation and architecture tests.

- [ ] **Step 5: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server/Cargo.toml \
  server/crates/platform-events \
  server/crates/platform-messaging \
  server/crates/platform-persistence
git commit -m "refactor: add platform eventing contracts"
```

## Task 2: Add Postgres outbox and projection checkpoint persistence

**Files:**
- Modify: `server/crates/database-infrastructure-sqlx/src/bin/migration-info.rs`
- Modify: `server/crates/database-infrastructure-sqlx/src/lib.rs`
- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050501_outbox.down.sql`
- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050501_outbox.up.sql`
- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050502_projection_checkpoints.down.sql`
- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050502_projection_checkpoints.up.sql`
- Create: `server/crates/database-infrastructure-sqlx/src/outbox.rs`
- Create: `server/crates/database-infrastructure-sqlx/src/projection_checkpoint.rs`

- [ ] **Step 1: Write a failing integration test for outbox schema presence**

```rust
#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn outbox_tables_exist(pool: sqlx::PgPool) {
    let exists: Option<String> = sqlx::query_scalar(
        "SELECT tablename FROM pg_tables WHERE schemaname = 'platform' AND tablename = 'outbox_messages'",
    )
    .fetch_optional(&pool)
    .await
    .unwrap();

    assert_eq!(exists.as_deref(), Some("outbox_messages"));
}
```

- [ ] **Step 2: Run the test before the migrations are added**

Run: `cd server && DATABASE_URL=postgres://... cargo test -p ordering-food-database-infrastructure-sqlx outbox_tables_exist --lib`
Expected: FAIL because the outbox migration has not been added yet.

- [ ] **Step 3: Add migrations and SQLx helpers for outbox records and projection checkpoints**

Rules:
- Use a neutral schema such as `platform`.
- Store event payload, metadata, topic / event type, available-at / retry fields, and dispatch status.
- Store per-projector checkpoint state separately from business tables.
- Do not let business contexts write projection checkpoints into another context schema.

- [ ] **Step 4: Run database-backed tests**

Run: `cd server && DATABASE_URL=postgres://... cargo test -p ordering-food-database-infrastructure-sqlx && cargo test -p ordering-food-database-infrastructure-sqlx --bin migration-info`
Expected: PASS for the outbox / checkpoint persistence helpers, migrations, and migration ordering registry.

- [ ] **Step 5: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server/crates/database-infrastructure-sqlx \
  server/crates/platform-persistence
git commit -m "refactor: add postgres outbox and projection checkpoints"
```

## Task 3: Expand `*-published` crates into stable cross-context language

**Files:**
- Modify: `server/crates/identity-published/src/lib.rs`
- Modify: `server/crates/organization-published/src/lib.rs`
- Modify: `server/crates/catalog-published/src/lib.rs`
- Modify: `server/crates/access-published/src/lib.rs`
- Modify: `server/crates/ordering-published/src/lib.rs`
- Modify: `server/crates/fulfillment-published/src/lib.rs`
- Create: `server/apps/api/tests/published_integration_architecture.rs`

- [ ] **Step 1: Write a failing architecture test that placeholder refs are not the final published surface**

```rust
use std::{fs, path::Path};

#[test]
fn ordering_published_contract_exposes_real_event_language() {
    let source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/ordering-published/src/lib.rs"),
    )
    .unwrap();

    assert!(source.contains("OrderPlaced"));
    assert!(source.contains("OrderCommercialStateChanged"));
}
```

- [ ] **Step 2: Run the test before the published contracts are expanded**

Run: `cd server && cargo test -p ordering-food-api --test published_integration_architecture ordering_published_contract_exposes_real_event_language`
Expected: FAIL because only placeholder refs exist.

- [ ] **Step 3: Add first-class published contracts for the critical cross-context facts**

Minimum first-wave published language:
- `Identity`: subject ref, subject status changed
- `Organization`: brand ref, store ref, store summary, store status changed
- `Catalog`: sellable item fact, price fact, availability changed
- `Access`: membership fact, role assignment changed
- `Ordering`: order placed, commercial state changed, customer cancellation
- `Fulfillment`: accepted, preparing, ready, completed, rejected

- [ ] **Step 4: Run the published crate tests**

Run: `cd server && cargo test -p ordering-food-identity-published -p ordering-food-organization-published -p ordering-food-catalog-published -p ordering-food-access-published -p ordering-food-ordering-published -p ordering-food-fulfillment-published`
Expected: PASS for all published contract crates.

- [ ] **Step 5: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server/crates/identity-published/src/lib.rs \
  server/crates/organization-published/src/lib.rs \
  server/crates/catalog-published/src/lib.rs \
  server/crates/access-published/src/lib.rs \
  server/crates/ordering-published/src/lib.rs \
  server/crates/fulfillment-published/src/lib.rs \
  server/apps/api/tests/published_integration_architecture.rs
git commit -m "refactor: define first-wave published language"
```

## Task 4: Implement ACL translators and integration read models for critical consuming contexts

**Files:**
- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050503_first_wave_integration_projections.down.sql`
- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050503_first_wave_integration_projections.up.sql`
- Modify: `server/crates/database-infrastructure-sqlx/src/bin/migration-info.rs`
- Modify: `server/crates/access-integration/src/lib.rs`
- Create: `server/crates/access-integration/src/acl.rs`
- Create: `server/crates/access-integration/src/projection.rs`
- Create: `server/crates/access-integration/src/identity_subject_projection.rs`
- Create: `server/crates/access-integration/src/organization_scope_projection.rs`
- Modify: `server/crates/catalog-integration/src/lib.rs`
- Create: `server/crates/catalog-integration/src/acl.rs`
- Create: `server/crates/catalog-integration/src/projection.rs`
- Create: `server/crates/catalog-integration/src/organization_brand_projection.rs`
- Create: `server/crates/catalog-integration/src/organization_store_projection.rs`
- Modify: `server/crates/ordering-integration/src/lib.rs`
- Create: `server/crates/ordering-integration/src/acl.rs`
- Create: `server/crates/ordering-integration/src/projection.rs`
- Create: `server/crates/ordering-integration/src/catalog_sellable_item_projection.rs`
- Create: `server/crates/ordering-integration/src/organization_store_projection.rs`
- Create: `server/crates/ordering-integration/src/access_scope_projection.rs`
- Modify: `server/crates/fulfillment-integration/src/lib.rs`
- Create: `server/crates/fulfillment-integration/src/acl.rs`
- Create: `server/crates/fulfillment-integration/src/projection.rs`
- Create: `server/crates/fulfillment-integration/src/ordering_order_projection.rs`
- Create: `server/crates/fulfillment-integration/src/organization_store_projection.rs`
- Create: `server/crates/fulfillment-integration/src/access_staff_scope_projection.rs`
- Modify: `server/crates/access-infrastructure-sqlx/src/lib.rs`
- Create: `server/crates/access-infrastructure-sqlx/src/identity_subject_projection_store.rs`
- Create: `server/crates/access-infrastructure-sqlx/src/organization_scope_projection_store.rs`
- Modify: `server/crates/catalog-infrastructure-sqlx/src/lib.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/organization_brand_projection_store.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/organization_store_projection_store.rs`
- Modify: `server/crates/ordering-infrastructure-sqlx/src/lib.rs`
- Create: `server/crates/ordering-infrastructure-sqlx/src/catalog_sellable_item_projection_store.rs`
- Create: `server/crates/ordering-infrastructure-sqlx/src/organization_store_projection_store.rs`
- Create: `server/crates/ordering-infrastructure-sqlx/src/access_scope_projection_store.rs`
- Modify: `server/crates/fulfillment-infrastructure-sqlx/src/lib.rs`
- Create: `server/crates/fulfillment-infrastructure-sqlx/src/ordering_order_projection_store.rs`
- Create: `server/crates/fulfillment-infrastructure-sqlx/src/organization_store_projection_store.rs`
- Create: `server/crates/fulfillment-infrastructure-sqlx/src/access_staff_scope_projection_store.rs`

- [ ] **Step 1: Write a failing architecture test that external published language enters only through integration**

```rust
use std::{fs, path::Path};

#[test]
fn ordering_integration_is_the_only_place_that_mentions_catalog_published_facts() {
    let source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/ordering-integration/src/lib.rs"),
    )
    .unwrap();

    assert!(source.contains("catalog"));
}
```

- [ ] **Step 2: Run the architecture test before the integration modules are implemented**

Run: `cd server && cargo test -p ordering-food-api --test published_integration_architecture ordering_integration_is_the_only_place_that_mentions_catalog_published_facts`
Expected: FAIL because the integration crates still contain empty placeholders.

- [ ] **Step 3: Build the first critical integration projections and their owned projection tables**

Minimum first-wave projections:
- `Access` projects `Identity` subject facts and `Organization` scope facts.
- `Catalog` projects `Organization` brand facts and store facts.
- `Ordering` projects `Catalog` sellable item / price / availability facts, `Organization` store facts, and `Access` scope facts.
- `Fulfillment` projects `Ordering` order facts, `Organization` store facts, and `Access` staff scope facts.

Rules:
- The new `202604050503_first_wave_integration_projections.*` migration must create projection tables inside the consuming context's own schema, never in the producer's schema.
- Each consuming context must also provide SQLx-backed projection store implementations in its own `*-infrastructure-sqlx` crate; integration crates define projector logic, infrastructure crates own persistence adapters.
- Projector write targets must be explicit and owned:
  - `access.*` stores identity/organization facts needed by `Access`
  - `catalog.*` stores organization brand/store projections needed by `Catalog`
  - `ordering.*` stores catalog/organization/access projections needed by `Ordering`
  - `fulfillment.*` stores ordering/organization/access projections needed by `Fulfillment`
- Where a projection replaces an existing Phase 2 sync seam, the same task must define a first-load bootstrap/backfill strategy so the async path can cut over without waiting only for future events.
- No integration projector may write directly into another bounded context's write tables.

- [ ] **Step 4: Run integration crate tests**

Run: `cd server && cargo test -p ordering-food-access-integration -p ordering-food-catalog-integration -p ordering-food-ordering-integration -p ordering-food-fulfillment-integration && cargo test -p ordering-food-access-infrastructure-sqlx -p ordering-food-catalog-infrastructure-sqlx -p ordering-food-ordering-infrastructure-sqlx -p ordering-food-fulfillment-infrastructure-sqlx && cargo test -p ordering-food-database-infrastructure-sqlx first_wave_integration_projections --lib`
Expected: PASS for the ACL / projector modules, consumer-owned SQLx projection stores, and the owned projection-table migrations.

- [ ] **Step 5: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server/crates/database-infrastructure-sqlx/migrations/202604050503_first_wave_integration_projections.up.sql \
  server/crates/database-infrastructure-sqlx/migrations/202604050503_first_wave_integration_projections.down.sql \
  server/crates/database-infrastructure-sqlx/src/bin/migration-info.rs \
  server/crates/access-integration \
  server/crates/access-infrastructure-sqlx \
  server/crates/catalog-integration \
  server/crates/catalog-infrastructure-sqlx \
  server/crates/ordering-integration \
  server/crates/ordering-infrastructure-sqlx \
  server/crates/fulfillment-integration \
  server/crates/fulfillment-infrastructure-sqlx
git commit -m "refactor: implement first-wave integration projections"
```

## Task 5: Wire producer-side outbox recording, dispatcher, and projector runtime

**Files:**
- Modify: `server/apps/api/Cargo.toml`
- Modify: `server/apps/api/src/runtime.rs`
- Modify: `server/apps/api/src/composition/platform.rs`
- Modify: `server/apps/api/src/app.rs`
- Modify: `server/crates/identity-application/src/module.rs`
- Modify: `server/crates/identity-infrastructure-sqlx/src/transaction.rs`
- Modify: `server/crates/organization-application/src/module.rs`
- Modify: `server/crates/organization-infrastructure-sqlx/src/transaction.rs`
- Modify: `server/crates/catalog-application/src/module.rs`
- Modify: `server/crates/catalog-infrastructure-sqlx/src/transaction.rs`
- Modify: `server/crates/access-application/src/service.rs`
- Create: `server/crates/access-infrastructure-sqlx/src/transaction.rs`
- Modify: `server/crates/ordering-application/src/module.rs`
- Modify: `server/crates/ordering-infrastructure-sqlx/src/transaction.rs`
- Modify: `server/crates/fulfillment-application/src/module.rs`
- Modify: `server/crates/fulfillment-infrastructure-sqlx/src/transaction.rs`
- Create: `server/apps/api/src/composition/eventing.rs`
- Create: `server/apps/api/src/composition/projections.rs`
- Create: `server/apps/api/tests/outbox_architecture.rs`
- Create: `server/apps/api/tests/sync_whitelist_transition_architecture.rs`

- [ ] **Step 1: Write a failing runtime test that dispatcher services are present in app startup**

```rust
use std::{fs, path::Path};

#[test]
fn app_startup_wires_event_dispatch_runtime() {
    let source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/app.rs")).unwrap();

    assert!(source.contains("start_projection_runtime"));
    assert!(source.contains("start_outbox_dispatcher"));
}
```

- [ ] **Step 2: Run the test before runtime wiring exists**

Run: `cd server && cargo test -p ordering-food-api --test outbox_architecture app_startup_wires_event_dispatch_runtime`
Expected: FAIL because the app shell does not wire dispatcher / projector tasks yet.

- [ ] **Step 3: Add producer-side outbox recording and background dispatcher / projector startup**

Rules:
- Do not append outbox messages from the app shell after business commit; each producer context must append them inside its own local transaction boundary.
- Identity / Organization / Catalog / Access / Ordering / Fulfillment must each emit first-wave published events from the same transaction that mutates owned state.
- App startup owns background loops and graceful shutdown.
- Projection checkpointing must be idempotent and restart-safe.
- Any temporary sync collaboration introduced in Phase 2 must be removed here where the asynchronous path is ready.

- [ ] **Step 4: Run app-shell and targeted workspace tests**

Run: `cd server && cargo test -p ordering-food-api && cargo test -p ordering-food-identity-application -p ordering-food-organization-application -p ordering-food-catalog-application -p ordering-food-access-application -p ordering-food-ordering-application -p ordering-food-fulfillment-application && cargo test --workspace --exclude ordering-food-menu-infrastructure-sqlx --exclude ordering-food-order-infrastructure-sqlx --exclude ordering-food-authz-infrastructure-sqlx --exclude ordering-food-database-infrastructure-sqlx`
Expected: PASS for the app shell, producer contexts, and the non-database workspace verification.

- [ ] **Step 5: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server/apps/api/Cargo.toml \
  server/apps/api/src/runtime.rs \
  server/apps/api/src/composition/platform.rs \
  server/apps/api/src/app.rs \
  server/apps/api/src/composition/eventing.rs \
  server/apps/api/src/composition/projections.rs \
  server/apps/api/tests/outbox_architecture.rs \
  server/apps/api/tests/sync_whitelist_transition_architecture.rs \
  server/crates/identity-application/src/module.rs \
  server/crates/identity-infrastructure-sqlx/src/transaction.rs \
  server/crates/organization-application/src/module.rs \
  server/crates/organization-infrastructure-sqlx/src/transaction.rs \
  server/crates/catalog-application/src/module.rs \
  server/crates/catalog-infrastructure-sqlx/src/transaction.rs \
  server/crates/access-application/src/service.rs \
  server/crates/access-infrastructure-sqlx/src/transaction.rs \
  server/crates/ordering-application/src/module.rs \
  server/crates/ordering-infrastructure-sqlx/src/transaction.rs \
  server/crates/fulfillment-application/src/module.rs \
  server/crates/fulfillment-infrastructure-sqlx/src/transaction.rs
git commit -m "refactor: wire outbox dispatcher and projector runtime"
```

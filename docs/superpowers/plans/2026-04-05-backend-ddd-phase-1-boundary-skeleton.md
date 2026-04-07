# Backend DDD Phase 1 Boundary Skeleton Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Introduce the first-phase backend DDD boundary skeleton without changing user-visible behavior: extract neutral platform abstractions, add target published/integration crates, add new context placeholder crates, and add architecture guards that prevent the codebase from sliding back to the old dependency shape.

**Architecture:** This phase is a non-breaking structural pass. Existing business logic remains in the current crates, while the workspace grows the target boundary skeleton needed by the approved architecture spec. The only live dependency-direction correction in this phase is removing the leaked `Clock` dependency path that currently routes through a business context and replacing it with a neutral platform crate.

**Tech Stack:** Rust 2024, Cargo workspace, Tokio, Axum, SQLx, thiserror, async-trait, cargo test / cargo nextest, architecture tests based on manifest/source scanning.

---

## Scope Check

The approved spec spans multiple independent subprojects. This plan intentionally covers only **Phase 1: boundary skeleton**. Do not mix in Phase 2-4 work here.

Follow-up plans will still be needed for:

1. Context redraw and business migration (`authz -> access`, `menu -> catalog`, `order -> ordering + fulfillment`)
2. Published language, ACL, and projector network
3. Outbox, dispatcher, projection rebuild, and sync-exception white-listing

## Planned File Map

### Existing files to modify

- `server/Cargo.toml`
  Register new workspace members for the Phase 1 crates.
- `server/apps/api/Cargo.toml`
  Add dependency on `ordering-food-platform-kernel`; do not add dependencies on new integration crates.
- `server/apps/api/src/runtime.rs`
  Move runtime clock implementation to the neutral platform contract and stop importing `Clock` from `ordering-food-identity-application`.
- `server/apps/api/src/composition/platform.rs`
  Replace business-owned clock trait usage with `ordering_food_platform_kernel::Clock`.
- `server/apps/api/src/composition/contexts/identity.rs`
  Wire platform clock into the identity module after the trait move.
- `server/apps/api/src/composition/contexts/menu.rs`
  Stop adapting `ordering_food_identity_application::Clock`; adapt the platform clock instead.
- `server/apps/api/src/composition/contexts/order.rs`
  Stop adapting `ordering_food_identity_application::Clock`; adapt the platform clock instead.
- `server/crates/identity-application/Cargo.toml`
  Add dependency on `ordering-food-platform-kernel`.
- `server/crates/identity-application/src/lib.rs`
  Re-export the neutral clock trait from `platform-kernel` instead of defining it locally.
- `server/crates/identity-application/src/ports.rs`
  Remove the local `Clock` trait definition; keep the existing user-specific ID generator for now.
- `server/crates/menu-application/Cargo.toml`
  Add dependency on `ordering-food-platform-kernel`.
- `server/crates/menu-application/src/lib.rs`
  Re-export the neutral clock trait from `platform-kernel`.
- `server/crates/menu-application/src/ports.rs`
  Remove the local `Clock` trait definition; leave menu-specific ID generation in place for Phase 2.
- `server/crates/order-application/Cargo.toml`
  Add dependency on `ordering-food-platform-kernel`.
- `server/crates/order-application/src/lib.rs`
  Re-export the neutral clock trait from `platform-kernel`.
- `server/crates/order-application/src/ports.rs`
  Remove the local `Clock` trait definition; leave order-specific ID generation in place for Phase 2.
- `server/apps/api/tests/architecture.rs`
  Extend existing app-shell guard coverage if needed, but do not overload it with every new rule.

### New files to create

#### Platform kernel

- `server/crates/platform-kernel/Cargo.toml`
- `server/crates/platform-kernel/src/lib.rs`
- `server/crates/platform-kernel/tests/architecture.rs`

#### App-shell architecture guards

- `server/apps/api/tests/platform_architecture.rs`
- `server/apps/api/tests/context_skeleton_architecture.rs`

#### Published crate skeletons

- `server/crates/identity-published/Cargo.toml`
- `server/crates/identity-published/src/lib.rs`
- `server/crates/catalog-published/Cargo.toml`
- `server/crates/catalog-published/src/lib.rs`
- `server/crates/ordering-published/Cargo.toml`
- `server/crates/ordering-published/src/lib.rs`
- `server/crates/access-published/Cargo.toml`
- `server/crates/access-published/src/lib.rs`
- `server/crates/organization-published/Cargo.toml`
- `server/crates/organization-published/src/lib.rs`
- `server/crates/fulfillment-published/Cargo.toml`
- `server/crates/fulfillment-published/src/lib.rs`

#### Integration crate skeletons

- `server/crates/identity-integration/Cargo.toml`
- `server/crates/identity-integration/src/lib.rs`
- `server/crates/catalog-integration/Cargo.toml`
- `server/crates/catalog-integration/src/lib.rs`
- `server/crates/ordering-integration/Cargo.toml`
- `server/crates/ordering-integration/src/lib.rs`
- `server/crates/access-integration/Cargo.toml`
- `server/crates/access-integration/src/lib.rs`
- `server/crates/organization-integration/Cargo.toml`
- `server/crates/organization-integration/src/lib.rs`
- `server/crates/fulfillment-integration/Cargo.toml`
- `server/crates/fulfillment-integration/src/lib.rs`

#### New context placeholder crates

- `server/crates/access-domain/Cargo.toml`
- `server/crates/access-domain/src/lib.rs`
- `server/crates/access-domain/tests/architecture.rs`
- `server/crates/access-application/Cargo.toml`
- `server/crates/access-application/src/lib.rs`
- `server/crates/organization-domain/Cargo.toml`
- `server/crates/organization-domain/src/lib.rs`
- `server/crates/organization-domain/tests/architecture.rs`
- `server/crates/organization-application/Cargo.toml`
- `server/crates/organization-application/src/lib.rs`
- `server/crates/fulfillment-domain/Cargo.toml`
- `server/crates/fulfillment-domain/src/lib.rs`
- `server/crates/fulfillment-domain/tests/architecture.rs`
- `server/crates/fulfillment-application/Cargo.toml`
- `server/crates/fulfillment-application/src/lib.rs`

## Task 1: Create the neutral platform-kernel crate

**Files:**
- Create: `server/crates/platform-kernel/Cargo.toml`
- Create: `server/crates/platform-kernel/src/lib.rs`
- Create: `server/crates/platform-kernel/tests/architecture.rs`
- Modify: `server/Cargo.toml`

- [ ] **Step 1: Write the failing architecture test for the new crate**

```rust
use std::{fs, path::Path};

#[test]
fn platform_kernel_manifest_stays_framework_free() {
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();

    for forbidden in ["axum", "sqlx", "redis", "tracing", "config", "serde_json", "anyhow"] {
        assert!(!manifest.contains(&format!("{forbidden}.workspace")));
        assert!(!manifest.contains(&format!("{forbidden} =")));
    }
}
```

- [ ] **Step 2: Run the test before the crate exists**

Run: `cd server && cargo test -p ordering-food-platform-kernel`

Expected: FAIL because the package does not exist in the workspace yet.

- [ ] **Step 3: Add the crate to the workspace and create the minimal implementation**

```toml
# server/crates/platform-kernel/Cargo.toml
[package]
name = "ordering-food-platform-kernel"
version.workspace = true
edition.workspace = true
license.workspace = true
publish.workspace = true

[dependencies]
ordering-food-shared-kernel = { path = "../shared-kernel" }
uuid.workspace = true

[dev-dependencies]
```

```rust
// server/crates/platform-kernel/src/lib.rs
use ordering_food_shared_kernel::Timestamp;

pub trait Clock: Send + Sync {
    fn now(&self) -> Timestamp;
}

pub trait UuidGenerator: Send + Sync {
    fn next_uuid(&self) -> uuid::Uuid;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CorrelationId(String);

impl CorrelationId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

- [ ] **Step 4: Run the crate tests and workspace metadata check**

Run: `cd server && cargo test -p ordering-food-platform-kernel && cargo metadata --no-deps >/tmp/ordering-food-metadata.json`

Expected: PASS for the crate test and successful metadata generation.

- [ ] **Step 5: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server/Cargo.toml \
  server/crates/platform-kernel/Cargo.toml \
  server/crates/platform-kernel/src/lib.rs \
  server/crates/platform-kernel/tests/architecture.rs
git commit -m "refactor: add neutral platform kernel crate"
```

## Task 2: Move clock ownership out of business contexts

**Files:**
- Modify: `server/apps/api/Cargo.toml`
- Modify: `server/apps/api/src/runtime.rs`
- Modify: `server/apps/api/src/composition/platform.rs`
- Modify: `server/apps/api/src/composition/contexts/identity.rs`
- Modify: `server/apps/api/src/composition/contexts/menu.rs`
- Modify: `server/apps/api/src/composition/contexts/order.rs`
- Modify: `server/crates/identity-application/Cargo.toml`
- Modify: `server/crates/identity-application/src/lib.rs`
- Modify: `server/crates/identity-application/src/ports.rs`
- Modify: `server/crates/menu-application/Cargo.toml`
- Modify: `server/crates/menu-application/src/lib.rs`
- Modify: `server/crates/menu-application/src/ports.rs`
- Modify: `server/crates/order-application/Cargo.toml`
- Modify: `server/crates/order-application/src/lib.rs`
- Modify: `server/crates/order-application/src/ports.rs`
- Create: `server/apps/api/tests/platform_architecture.rs`

- [ ] **Step 1: Write a failing architecture test that forbids local clock traits in application crates**

```rust
use std::{fs, path::Path};

#[test]
fn application_ports_do_not_define_clock_trait_locally() {
    for relative_path in [
        "../../crates/identity-application/src/ports.rs",
        "../../crates/menu-application/src/ports.rs",
        "../../crates/order-application/src/ports.rs",
    ] {
        let source =
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)).unwrap();
        assert!(!source.contains("pub trait Clock"));
    }
}
```

- [ ] **Step 2: Run the new architecture test**

Run: `cd server && cargo test -p ordering-food-api --test platform_architecture`

Expected: FAIL because all three application crates currently define `pub trait Clock`.

- [ ] **Step 3: Update the application crates to re-export the neutral clock trait**

```rust
// identity-application/src/lib.rs
pub use ordering_food_platform_kernel::Clock;
```

```rust
// identity-application/src/ports.rs
use ordering_food_platform_kernel::Clock;
```

```rust
// apps/api/src/runtime.rs
use ordering_food_platform_kernel::Clock;
```

Notes:
- Remove the local `Clock` trait definitions from `identity-application`, `menu-application`, and `order-application`.
- Keep aggregate-specific ID generators in place for Phase 1; do not try to fully normalize all ID factories yet.
- Replace `ordering_food_identity_application::Clock` imports in `menu` and `order` adapters with `ordering_food_platform_kernel::Clock`.

- [ ] **Step 4: Run targeted tests and one compile-heavy workspace pass**

Run: `cd server && cargo test -p ordering-food-api --test platform_architecture && cargo test -p ordering-food-api runtime::tests::uuid_v4_user_id_generator_generates_uuid_v4_string --lib && cargo test -p ordering-food-identity-application && cargo test -p ordering-food-menu-application && cargo test -p ordering-food-order-application`

Expected: PASS for the new architecture test and all affected package tests.

- [ ] **Step 5: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server/apps/api/Cargo.toml \
  server/apps/api/src/runtime.rs \
  server/apps/api/src/composition/platform.rs \
  server/apps/api/src/composition/contexts/identity.rs \
  server/apps/api/src/composition/contexts/menu.rs \
  server/apps/api/src/composition/contexts/order.rs \
  server/apps/api/tests/platform_architecture.rs \
  server/crates/identity-application/Cargo.toml \
  server/crates/identity-application/src/lib.rs \
  server/crates/identity-application/src/ports.rs \
  server/crates/menu-application/Cargo.toml \
  server/crates/menu-application/src/lib.rs \
  server/crates/menu-application/src/ports.rs \
  server/crates/order-application/Cargo.toml \
  server/crates/order-application/src/lib.rs \
  server/crates/order-application/src/ports.rs
git commit -m "refactor: move clock ownership to platform kernel"
```

## Task 3: Add published crate skeletons for target contexts

**Files:**
- Modify: `server/Cargo.toml`
- Create: `server/crates/identity-published/Cargo.toml`
- Create: `server/crates/identity-published/src/lib.rs`
- Create: `server/crates/catalog-published/Cargo.toml`
- Create: `server/crates/catalog-published/src/lib.rs`
- Create: `server/crates/ordering-published/Cargo.toml`
- Create: `server/crates/ordering-published/src/lib.rs`
- Create: `server/crates/access-published/Cargo.toml`
- Create: `server/crates/access-published/src/lib.rs`
- Create: `server/crates/organization-published/Cargo.toml`
- Create: `server/crates/organization-published/src/lib.rs`
- Create: `server/crates/fulfillment-published/Cargo.toml`
- Create: `server/crates/fulfillment-published/src/lib.rs`
- Create: `server/apps/api/tests/context_skeleton_architecture.rs`

- [ ] **Step 1: Write a failing workspace skeleton test**

```rust
use std::{fs, path::Path};

#[test]
fn workspace_members_include_target_published_crates() {
    let manifest = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml"),
    )
    .unwrap();

    for member in [
        "crates/identity-published",
        "crates/catalog-published",
        "crates/ordering-published",
        "crates/access-published",
        "crates/organization-published",
        "crates/fulfillment-published",
    ] {
        assert!(manifest.contains(member), "missing workspace member: {member}");
    }
}
```

- [ ] **Step 2: Run the architecture test before the members exist**

Run: `cd server && cargo test -p ordering-food-api --test context_skeleton_architecture workspace_members_include_target_published_crates`

Expected: FAIL because the new published crates are not listed yet.

- [ ] **Step 3: Create the published crates with minimal, compiling placeholders**

```rust
//! Published contracts for the identity bounded context.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectRef {
    pub subject_id: String,
}
```

Use the same pattern for the other published crates:
- `Catalog` gets a placeholder like `SellableItemRef`
- `Ordering` gets `OrderRef`
- `Access` gets `MembershipRef`
- `Organization` gets `StoreRef`
- `Fulfillment` gets `FulfillmentOrderRef`

Keep these placeholders intentionally small; this phase establishes crate ownership, not final event design.

- [ ] **Step 4: Run the new test and compile the placeholder crates**

Run: `cd server && cargo test -p ordering-food-api --test context_skeleton_architecture workspace_members_include_target_published_crates && cargo test -p ordering-food-identity-published && cargo test -p ordering-food-catalog-published && cargo test -p ordering-food-ordering-published && cargo test -p ordering-food-access-published && cargo test -p ordering-food-organization-published && cargo test -p ordering-food-fulfillment-published`

Expected: PASS across the architecture test and all published packages.

- [ ] **Step 5: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server/Cargo.toml \
  server/apps/api/tests/context_skeleton_architecture.rs \
  server/crates/identity-published \
  server/crates/catalog-published \
  server/crates/ordering-published \
  server/crates/access-published \
  server/crates/organization-published \
  server/crates/fulfillment-published
git commit -m "refactor: add published crate skeletons"
```

## Task 4: Add integration crate skeletons for target contexts

**Files:**
- Modify: `server/Cargo.toml`
- Create: `server/crates/identity-integration/Cargo.toml`
- Create: `server/crates/identity-integration/src/lib.rs`
- Create: `server/crates/catalog-integration/Cargo.toml`
- Create: `server/crates/catalog-integration/src/lib.rs`
- Create: `server/crates/ordering-integration/Cargo.toml`
- Create: `server/crates/ordering-integration/src/lib.rs`
- Create: `server/crates/access-integration/Cargo.toml`
- Create: `server/crates/access-integration/src/lib.rs`
- Create: `server/crates/organization-integration/Cargo.toml`
- Create: `server/crates/organization-integration/src/lib.rs`
- Create: `server/crates/fulfillment-integration/Cargo.toml`
- Create: `server/crates/fulfillment-integration/src/lib.rs`

- [ ] **Step 1: Extend the workspace skeleton test to require integration crates**

```rust
for member in [
    "crates/identity-integration",
    "crates/catalog-integration",
    "crates/ordering-integration",
    "crates/access-integration",
    "crates/organization-integration",
    "crates/fulfillment-integration",
] {
    assert!(manifest.contains(member), "missing workspace member: {member}");
}
```

- [ ] **Step 2: Run the existing architecture test again**

Run: `cd server && cargo test -p ordering-food-api --test context_skeleton_architecture`

Expected: FAIL because the integration crates do not exist yet.

- [ ] **Step 3: Create each integration crate with explicit ACL / projection placeholder modules**

```rust
pub mod acl {
    pub trait ExternalFactTranslator {}
}

pub mod projection {
    pub trait ProjectionUpdater {}
}
```

Notes:
- Keep the traits empty in Phase 1.
- The point is to make `integration` physically real and searchable.

- [ ] **Step 4: Run package tests for all integration crates**

Run: `cd server && cargo test -p ordering-food-identity-integration && cargo test -p ordering-food-catalog-integration && cargo test -p ordering-food-ordering-integration && cargo test -p ordering-food-access-integration && cargo test -p ordering-food-organization-integration && cargo test -p ordering-food-fulfillment-integration`

Expected: PASS for all integration crates.

- [ ] **Step 5: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server/Cargo.toml \
  server/apps/api/tests/context_skeleton_architecture.rs \
  server/crates/identity-integration \
  server/crates/catalog-integration \
  server/crates/ordering-integration \
  server/crates/access-integration \
  server/crates/organization-integration \
  server/crates/fulfillment-integration
git commit -m "refactor: add integration crate skeletons"
```

## Task 5: Add placeholder domain and application crates for new contexts

**Files:**
- Modify: `server/Cargo.toml`
- Create: `server/crates/access-domain/Cargo.toml`
- Create: `server/crates/access-domain/src/lib.rs`
- Create: `server/crates/access-domain/tests/architecture.rs`
- Create: `server/crates/access-application/Cargo.toml`
- Create: `server/crates/access-application/src/lib.rs`
- Create: `server/crates/organization-domain/Cargo.toml`
- Create: `server/crates/organization-domain/src/lib.rs`
- Create: `server/crates/organization-domain/tests/architecture.rs`
- Create: `server/crates/organization-application/Cargo.toml`
- Create: `server/crates/organization-application/src/lib.rs`
- Create: `server/crates/fulfillment-domain/Cargo.toml`
- Create: `server/crates/fulfillment-domain/src/lib.rs`
- Create: `server/crates/fulfillment-domain/tests/architecture.rs`
- Create: `server/crates/fulfillment-application/Cargo.toml`
- Create: `server/crates/fulfillment-application/src/lib.rs`

- [ ] **Step 1: Write a failing domain architecture test for one new context and copy the pattern**

```rust
use std::{fs, path::Path};

#[test]
fn domain_manifest_does_not_depend_on_framework_or_infrastructure_crates() {
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();

    for forbidden in ["axum", "sqlx", "redis", "tracing", "config", "serde_json", "anyhow"] {
        assert!(!manifest.contains(&format!("{forbidden}.workspace")));
        assert!(!manifest.contains(&format!("{forbidden} =")));
    }
}
```

- [ ] **Step 2: Run the package test before the package exists**

Run: `cd server && cargo test -p ordering-food-access-domain`

Expected: FAIL because the package does not exist yet.

- [ ] **Step 3: Create the new domain/application placeholders**

```rust
// access-domain/src/lib.rs
//! Domain skeleton for the Access bounded context.

pub struct AccessContextBoundary;
```

```rust
// access-application/src/lib.rs
//! Application skeleton for the Access bounded context.

pub struct AccessApplicationBoundary;
```

Repeat the same structure for `organization-*` and `fulfillment-*`.

Notes:
- Keep them intentionally small.
- Do not migrate current `authz` behavior in this phase.
- Do not implement use cases yet.

- [ ] **Step 4: Run tests for the new packages**

Run: `cd server && cargo test -p ordering-food-access-domain && cargo test -p ordering-food-access-application && cargo test -p ordering-food-organization-domain && cargo test -p ordering-food-organization-application && cargo test -p ordering-food-fulfillment-domain && cargo test -p ordering-food-fulfillment-application`

Expected: PASS for all new context placeholder crates.

- [ ] **Step 5: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server/Cargo.toml \
  server/crates/access-domain \
  server/crates/access-application \
  server/crates/organization-domain \
  server/crates/organization-application \
  server/crates/fulfillment-domain \
  server/crates/fulfillment-application
git commit -m "refactor: add target context placeholder crates"
```

## Task 6: Add Phase 1 boundary guardrails to the app shell

**Files:**
- Modify: `server/apps/api/tests/architecture.rs`
- Modify: `server/apps/api/tests/context_skeleton_architecture.rs`

- [ ] **Step 1: Add a failing guard that forbids the app shell from depending on integration crates**

```rust
#[test]
fn api_manifest_does_not_depend_on_target_integration_crates() {
    let manifest =
        std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .unwrap();

    for forbidden in [
        "ordering-food-identity-integration",
        "ordering-food-catalog-integration",
        "ordering-food-ordering-integration",
        "ordering-food-access-integration",
        "ordering-food-organization-integration",
        "ordering-food-fulfillment-integration",
    ] {
        assert!(!manifest.contains(forbidden), "forbidden dependency: {forbidden}");
    }
}
```

- [ ] **Step 2: Run the app-shell architecture suite**

Run: `cd server && cargo test -p ordering-food-api --test architecture && cargo test -p ordering-food-api --test context_skeleton_architecture && cargo test -p ordering-food-api --test platform_architecture`

Expected: FAIL if any skeleton crate was wired into the app shell incorrectly.

- [ ] **Step 3: Fix any manifest wiring so the app shell still points only to active implementation crates**

```toml
# apps/api/Cargo.toml should keep using live implementation crates only.
ordering-food-identity-application = { path = "../../crates/identity-application" }
ordering-food-menu-application = { path = "../../crates/menu-application" }
ordering-food-order-application = { path = "../../crates/order-application" }
ordering-food-authz-application = { path = "../../crates/authz-application" }
ordering-food-platform-kernel = { path = "../../crates/platform-kernel" }
```

Do not add `*-integration` or new placeholder `*-application` crates to `apps/api` in Phase 1.

- [ ] **Step 4: Run the full Phase 1 verification set**

Run: `cd server && cargo test -p ordering-food-api --test architecture && cargo test -p ordering-food-api --test platform_architecture && cargo test -p ordering-food-api --test context_skeleton_architecture && cargo test -p ordering-food-platform-kernel && cargo test -p ordering-food-access-domain && cargo test -p ordering-food-organization-domain && cargo test -p ordering-food-fulfillment-domain`

Expected: PASS across all guards and new skeleton packages.

- [ ] **Step 5: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server/apps/api/tests/architecture.rs \
  server/apps/api/tests/platform_architecture.rs \
  server/apps/api/tests/context_skeleton_architecture.rs \
  server/apps/api/Cargo.toml
git commit -m "test: guard app shell against boundary regressions"
```

## Task 7: Run a final workspace-level regression pass for Phase 1

**Files:**
- Modify: none
- Test: all files changed in Tasks 1-6

- [ ] **Step 1: Run formatting check**

Run: `cd server && cargo fmt --all --check`

Expected: PASS or actionable formatting diffs only.

- [ ] **Step 2: Run targeted clippy on the touched packages**

Run: `cd server && cargo clippy -p ordering-food-api -p ordering-food-platform-kernel -p ordering-food-identity-application -p ordering-food-menu-application -p ordering-food-order-application -p ordering-food-access-domain -p ordering-food-organization-domain -p ordering-food-fulfillment-domain --tests -- -D warnings`

Expected: PASS with no warnings.

- [ ] **Step 3: Run targeted test matrix**

Run: `cd server && cargo test -p ordering-food-api --tests && cargo test -p ordering-food-platform-kernel && cargo test -p ordering-food-identity-published && cargo test -p ordering-food-catalog-published && cargo test -p ordering-food-ordering-published && cargo test -p ordering-food-access-published && cargo test -p ordering-food-organization-published && cargo test -p ordering-food-fulfillment-published && cargo test -p ordering-food-identity-integration && cargo test -p ordering-food-catalog-integration && cargo test -p ordering-food-ordering-integration && cargo test -p ordering-food-access-integration && cargo test -p ordering-food-organization-integration && cargo test -p ordering-food-fulfillment-integration`

Expected: PASS.

- [ ] **Step 4: Record Phase 1 completion notes**

```text
- Existing behavior unchanged
- Neutral platform clock path in place
- Target crate skeletons present
- App shell still isolated from integration crates
- Context redraw and event network deferred to later plans
```

- [ ] **Step 5: Commit if and only if the user explicitly authorizes git commits**

```bash
git add server
git commit -m "refactor: establish backend ddd boundary skeleton"
```

## Execution Notes

- Do not rename `authz`, `menu`, or `order` crates in this plan.
- Do not migrate business logic into the new placeholder crates in this plan.
- Do not add external message brokers in this plan.
- Do not introduce `published event` payload completeness in this plan; only establish physical ownership.
- If a task requires line-range updates not known in advance, locate the exact ranges before editing and keep the change set scoped to the files listed here.

## Handoff

After Phase 1 is complete, the next plan should cover **Phase 2: context redraw and business migration**.

# Backend DDD Phase 2C Catalog Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将当前 `menu` 上下文迁移为 `Catalog` 语义和物理结构：把现有“门店即菜单真相”的模型改写为“品牌目录 + 门店可售目录 + 分类 + 菜品 + 定价 + 可售状态 + 展示规则”的边界，同时完成 `server` workspace 内从 `menu-*` 到 `catalog-*` 的代码、路由、SQL schema 与测试切换。

**Architecture:** 本阶段采用“并行建立 `catalog-*` crate -> 完成领域语义重画 -> 切换 `apps/api` -> 清理 `menu-*` 遗留”的迁移方式，而不是直接原地改旧 crate。`Catalog` 只拥有目录结构、价格、可售状态和展示规则；`Brand` / `Store` 不再由目录上下文声明真相，而是作为 `Organization` 发布的 scope facts 被 `catalog-application` 消费，并在 `catalog` 自己的持久化模型中仅保留本地所需引用与快照字段。`ACL`、`outbox`、`event projector`、异步投影网络只保留明确命名的接口与占位，不在本阶段做成完整网络。

**Tech Stack:** Rust 2024, Cargo workspace, Tokio, Axum, SQLx, PostgreSQL migration files, utoipa, ts-rs, `cargo test`, `cargo clippy`, 基于源码扫描的 architecture tests。

---

## Planned File Map

### Workspace 与 API 壳层

- Modify: `server/Cargo.toml`
  先并行加入 `catalog-domain`、`catalog-application`、`catalog-infrastructure-sqlx`，最终移除 `menu-domain`、`menu-application`、`menu-infrastructure-sqlx`。
- Modify: `server/apps/api/Cargo.toml`
  将 API 依赖从 `ordering-food-menu-*` 切换到 `ordering-food-catalog-*`，并引入 `ordering-food-organization-published` 以承载稳定的 scope fact 适配。
- Modify: `server/apps/api/src/config.rs`
  增加单品牌运行期所需的本地默认 brand scope 配置，例如 `CATALOG__BOOTSTRAP__BRAND_ID`、`CATALOG__BOOTSTRAP__BRAND_SLUG`、`CATALOG__BOOTSTRAP__BRAND_NAME`；该配置不得覆盖 `Organization` 已持久化 identity。
- Create: `server/apps/api/src/composition/contexts/catalog.rs`
  以 `catalog` 命名重新接入目录上下文，组合 `CatalogModule`、`Organization` published scope 适配器和默认 seed。
- Modify: `server/apps/api/src/composition/contexts/mod.rs`
  用 `catalog::register_catalog()` 替换 `menu::register_menu()`。
- Modify: `server/apps/api/src/composition/registry.rs`
  将测试和上下文 ID 从 `menu` 改为 `catalog`。
- Create: `server/apps/api/src/routes/catalog.rs`
  用 `Catalog*` DTO、`/api/catalog` 前缀和 `CatalogModule` 替换现有 `menu` 路由文件。
- Modify: `server/apps/api/src/routes/mod.rs`
  暴露 `catalog` 路由模块并移除 `menu` 模块注册。
- Modify: `server/apps/api/src/ts_bindings.rs`
  将导出的前端契约类型从 `Menu*` 切换为 `Catalog*`。
- Create: `server/apps/api/tests/catalog_architecture.rs`
  新增 `catalog` 边界守卫，覆盖路由隔离、前缀命名、上下文切换。
- Modify: `server/apps/api/tests/architecture.rs`
  将新的 `src/routes/catalog.rs` 纳入 API 架构测试路径列表。
- Modify: `server/apps/api/tests/order_architecture.rs`
  将新的 `src/routes/catalog.rs` 纳入订单侧架构测试路径列表。
- Modify: `server/apps/api/tests/platform_architecture.rs`
  将 `../../crates/menu-application/src/ports.rs` 替换为 `../../crates/catalog-application/src/ports.rs`。
- Modify: `server/apps/api/tests/context_skeleton_architecture.rs`
  先要求 workspace 存在 `catalog-*` 业务 crate，最终要求 `menu-*` 业务 crate 不再保留。

### 新的 Catalog 领域 / 应用 / 持久化 crate

- Create: `server/crates/catalog-domain/Cargo.toml`
- Create: `server/crates/catalog-domain/src/lib.rs`
- Create: `server/crates/catalog-domain/src/error.rs`
- Create: `server/crates/catalog-domain/src/brand_id.rs`
- Create: `server/crates/catalog-domain/src/brand_catalog_id.rs`
- Create: `server/crates/catalog-domain/src/brand_catalog.rs`
- Create: `server/crates/catalog-domain/src/store_id.rs`
- Create: `server/crates/catalog-domain/src/store_catalog_id.rs`
- Create: `server/crates/catalog-domain/src/store_catalog.rs`
- Create: `server/crates/catalog-domain/src/category_id.rs`
- Create: `server/crates/catalog-domain/src/category.rs`
- Create: `server/crates/catalog-domain/src/item_id.rs`
- Create: `server/crates/catalog-domain/src/item.rs`
- Create: `server/crates/catalog-domain/src/store_item_listing.rs`
- Create: `server/crates/catalog-domain/src/price.rs`
- Create: `server/crates/catalog-domain/src/sellable_status.rs`
- Create: `server/crates/catalog-domain/src/display_rule.rs`
- Create: `server/crates/catalog-domain/tests/architecture.rs`

- Create: `server/crates/catalog-application/Cargo.toml`
- Create: `server/crates/catalog-application/src/lib.rs`
- Create: `server/crates/catalog-application/src/error.rs`
- Create: `server/crates/catalog-application/src/dto.rs`
- Create: `server/crates/catalog-application/src/module.rs`
- Create: `server/crates/catalog-application/src/ports.rs`
- Create: `server/crates/catalog-application/src/use_cases/mod.rs`
- Create: `server/crates/catalog-application/src/use_cases/bootstrap_brand_catalog.rs`
- Create: `server/crates/catalog-application/src/use_cases/attach_store_catalog.rs`
- Create: `server/crates/catalog-application/src/use_cases/create_category.rs`
- Create: `server/crates/catalog-application/src/use_cases/create_item.rs`
- Create: `server/crates/catalog-application/src/use_cases/upsert_store_item_listing.rs`
- Create: `server/crates/catalog-application/tests/architecture.rs`

- Create: `server/crates/catalog-infrastructure-sqlx/Cargo.toml`
- Create: `server/crates/catalog-infrastructure-sqlx/src/lib.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/module.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/brand_catalog_repository.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/store_catalog_repository.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/category_repository.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/item_repository.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/store_item_listing_repository.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/brand_catalog_read_repository.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/store_catalog_read_repository.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/category_read_repository.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/item_read_repository.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/transaction.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/tests/architecture.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/tests/repositories.rs`

### Published / Integration / Organization scope contracts

- Modify: `server/crates/catalog-published/src/lib.rs`
  把 Phase 1 的 `SellableItemRef` 占位升级为 `CatalogItemRef`、`CatalogPriceFact`、`StoreCatalogRef` 等稳定对外语言。
- Modify: `server/crates/catalog-integration/Cargo.toml`
  加入对 `ordering-food-organization-published` 的依赖。
- Modify: `server/crates/catalog-integration/src/lib.rs`
  将泛化 `acl` / `projection` 占位改成与 `Organization` scope 事实明确对应的 adapter / translator 命名。
- Modify: `server/crates/organization-published/src/lib.rs`
  沿用 `Phase 2A` 已建立的 `BrandRef`、`StoreRef`、`StoreSummary` 作为稳定 scope facts，供 `Catalog` 消费；本阶段不再引入平行命名。

### 数据迁移

- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050301_catalog_context.up.sql`
  新建 `catalog` schema、品牌目录表、门店可售目录表、分类表、菜品表、门店菜品 listing 表，并从 `menu.*` 回填初始数据。
- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050301_catalog_context.down.sql`
  提供从 `catalog` 回退到只保留旧结构的数据库回滚路径。
- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050302_drop_menu_context.up.sql`
  在代码与测试完全切换后删除 `menu` schema 和其索引 / 约束。
- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050302_drop_menu_context.down.sql`
  为最后一步 schema 清理提供显式回退脚本。

### 最终删除的旧 `menu` 文件

- Delete: `server/crates/menu-domain/Cargo.toml`
- Delete: `server/crates/menu-domain/src/category.rs`
- Delete: `server/crates/menu-domain/src/category_id.rs`
- Delete: `server/crates/menu-domain/src/error.rs`
- Delete: `server/crates/menu-domain/src/item.rs`
- Delete: `server/crates/menu-domain/src/item_id.rs`
- Delete: `server/crates/menu-domain/src/lib.rs`
- Delete: `server/crates/menu-domain/src/status.rs`
- Delete: `server/crates/menu-domain/src/store.rs`
- Delete: `server/crates/menu-domain/src/store_id.rs`
- Delete: `server/crates/menu-application/Cargo.toml`
- Delete: `server/crates/menu-application/src/dto.rs`
- Delete: `server/crates/menu-application/src/error.rs`
- Delete: `server/crates/menu-application/src/lib.rs`
- Delete: `server/crates/menu-application/src/module.rs`
- Delete: `server/crates/menu-application/src/ports.rs`
- Delete: `server/crates/menu-application/src/use_cases/create_category.rs`
- Delete: `server/crates/menu-application/src/use_cases/create_item.rs`
- Delete: `server/crates/menu-application/src/use_cases/create_store.rs`
- Delete: `server/crates/menu-application/src/use_cases/mod.rs`
- Delete: `server/crates/menu-infrastructure-sqlx/Cargo.toml`
- Delete: `server/crates/menu-infrastructure-sqlx/src/category_read_repository.rs`
- Delete: `server/crates/menu-infrastructure-sqlx/src/category_repository.rs`
- Delete: `server/crates/menu-infrastructure-sqlx/src/item_read_repository.rs`
- Delete: `server/crates/menu-infrastructure-sqlx/src/item_repository.rs`
- Delete: `server/crates/menu-infrastructure-sqlx/src/lib.rs`
- Delete: `server/crates/menu-infrastructure-sqlx/src/module.rs`
- Delete: `server/crates/menu-infrastructure-sqlx/src/store_read_repository.rs`
- Delete: `server/crates/menu-infrastructure-sqlx/src/store_repository.rs`
- Delete: `server/crates/menu-infrastructure-sqlx/src/transaction.rs`
- Delete: `server/crates/menu-infrastructure-sqlx/tests/repositories.rs`
- Delete: `server/apps/api/src/composition/contexts/menu.rs`
- Delete: `server/apps/api/src/routes/menu.rs`
- Delete: `server/apps/api/tests/menu_architecture.rs`

## Scope Check

- 本计划只覆盖 `server` workspace，不包含 `frontend` 的契约适配或页面改造。
- 本计划不重写 `server/crates/database-infrastructure-sqlx/migrations/202603140001_baseline.up.sql` 与 `server/crates/database-infrastructure-sqlx/migrations/202603140001_baseline.down.sql`；数据库迁移必须通过新的 forward migration 完成，避免改写既有基线。
- 本计划不实现完整的 `ACL`、`outbox`、`event projector`、异步投影重建机制；这里只定义清晰的消费点、命名和端口。
- 本计划不处理 `order` / `ordering` 侧的语义升级，因此 `server/crates/order-domain/src/menu_item_id.rs` 这一遗留命名暂时保留，等 `Ordering` / `Fulfillment` 计划单独处理。
- 本计划默认单品牌运行仍然成立，因此数据库回填与 API seed 可以保留一个 canonical brand scope 的配置入口；但该配置只能作为测试/本地默认值，真实品牌与门店 identity 必须以前置 `Phase 2A` 已落地的 `Organization` published language 为准，不能覆盖已持久化的 `organization.brands` / `organization.stores` 真相。
- 本计划不保留 `/api/menu` 双写兼容路径；如果执行时确认必须兼容旧 API，应拆出单独的 API 兼容计划，而不是把两套术语长期并存到同一个 phase 中。

## Task 1: 建立 Catalog 业务 crate 骨架并锁定 workspace 迁移目标

**Files:**
- Create: `server/crates/catalog-domain/Cargo.toml`
- Create: `server/crates/catalog-domain/src/lib.rs`
- Create: `server/crates/catalog-domain/tests/architecture.rs`
- Create: `server/crates/catalog-application/Cargo.toml`
- Create: `server/crates/catalog-application/src/lib.rs`
- Create: `server/crates/catalog-application/tests/architecture.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/Cargo.toml`
- Create: `server/crates/catalog-infrastructure-sqlx/src/lib.rs`
- Modify: `server/Cargo.toml`
- Modify: `server/apps/api/tests/context_skeleton_architecture.rs`

- [ ] **Step 1: 先写一个失败的 workspace 守卫测试**

```rust
#[test]
fn workspace_members_include_catalog_business_crates() {
    let manifest = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml"),
    )
    .unwrap();

    for member in [
        "crates/catalog-domain",
        "crates/catalog-application",
        "crates/catalog-infrastructure-sqlx",
    ] {
        assert!(manifest.contains(member), "missing workspace member: {member}");
    }
}
```

- [ ] **Step 2: 运行测试确认当前确实失败**

Run: `cd server && cargo test -p ordering-food-api --test context_skeleton_architecture`

Expected: FAIL，提示 `catalog-domain`、`catalog-application` 或 `catalog-infrastructure-sqlx` 还不在 workspace 中。

- [ ] **Step 3: 创建三个新 crate 的最小骨架，并把它们加入 workspace**

```toml
# server/crates/catalog-domain/Cargo.toml
[package]
name = "ordering-food-catalog-domain"
version.workspace = true
edition.workspace = true
license.workspace = true
publish.workspace = true

[dependencies]
ordering-food-shared-kernel = { path = "../shared-kernel" }
thiserror.workspace = true
time.workspace = true
```

```rust
// server/crates/catalog-domain/src/lib.rs
//! Domain model for the Catalog bounded context.
```

- [ ] **Step 4: 运行最小 package 测试和 workspace 守卫**

Run: `cd server && cargo test -p ordering-food-catalog-domain && cargo test -p ordering-food-catalog-application && cargo test -p ordering-food-catalog-infrastructure-sqlx && cargo test -p ordering-food-api --test context_skeleton_architecture`

Expected: PASS，说明新的业务 crate 已经可编译，且 Phase 2C 的物理迁移入口已经建立。

- [ ] **Step 5: 仅当用户明确授权时，提交当前任务变更**

```bash
git add server/Cargo.toml \
  server/apps/api/tests/context_skeleton_architecture.rs \
  server/crates/catalog-domain \
  server/crates/catalog-application \
  server/crates/catalog-infrastructure-sqlx
git commit -m "refactor: add catalog business crate skeletons"
```

## Task 2: 重画 Catalog 领域语义，移除 `Store` 组织真相所有权

**Files:**
- Modify: `server/crates/catalog-domain/src/lib.rs`
- Create: `server/crates/catalog-domain/src/error.rs`
- Create: `server/crates/catalog-domain/src/brand_id.rs`
- Create: `server/crates/catalog-domain/src/brand_catalog_id.rs`
- Create: `server/crates/catalog-domain/src/brand_catalog.rs`
- Create: `server/crates/catalog-domain/src/store_id.rs`
- Create: `server/crates/catalog-domain/src/store_catalog_id.rs`
- Create: `server/crates/catalog-domain/src/store_catalog.rs`
- Create: `server/crates/catalog-domain/src/category_id.rs`
- Create: `server/crates/catalog-domain/src/category.rs`
- Create: `server/crates/catalog-domain/src/item_id.rs`
- Create: `server/crates/catalog-domain/src/item.rs`
- Create: `server/crates/catalog-domain/src/store_item_listing.rs`
- Create: `server/crates/catalog-domain/src/price.rs`
- Create: `server/crates/catalog-domain/src/sellable_status.rs`
- Create: `server/crates/catalog-domain/src/display_rule.rs`
- Modify: `server/crates/catalog-domain/tests/architecture.rs`

- [ ] **Step 1: 写领域测试，先把目标语义钉死**

```rust
#[test]
fn store_catalog_requires_external_brand_and_store_scope() {
    let now = time::macros::datetime!(2026-04-05 10:00 UTC);
    let catalog = StoreCatalog::attach(
        StoreCatalogId::new("store-catalog-1"),
        BrandId::new("brand-1"),
        StoreId::new("store-1"),
        SellableStatus::Sellable,
        DisplayRule::listed(),
        now,
    )
    .unwrap();

    assert_eq!(catalog.brand_id().as_str(), "brand-1");
    assert_eq!(catalog.store_id().as_str(), "store-1");
}
```

- [ ] **Step 2: 运行领域 crate，确认新模型尚未存在**

Run: `cd server && cargo test -p ordering-food-catalog-domain`

Expected: FAIL，报错集中在 `StoreCatalog`、`BrandCatalog`、`SellableStatus`、`DisplayRule`、`Price` 等类型不存在。

- [ ] **Step 3: 实现新的 Catalog 领域模型，不再保留 `CreateStore` 式的目录内组织所有权**

```rust
pub struct BrandCatalog {
    id: BrandCatalogId,
    brand_id: BrandId,
    slug: String,
    name: String,
}

pub struct StoreCatalog {
    id: StoreCatalogId,
    brand_catalog_id: BrandCatalogId,
    brand_id: BrandId,
    store_id: StoreId,
    status: SellableStatus,
    display_rule: DisplayRule,
}
```

- [ ] **Step 4: 补一个 architecture test，禁止 `catalog-domain` 直接依赖 `Organization` published 或任何技术实现**

Run: `cd server && cargo test -p ordering-food-catalog-domain --test architecture`

Expected: PASS，且测试内容至少覆盖以下规则：
- `catalog-domain` 不出现 `sqlx`、`axum`、`redis`
- `catalog-domain` 不出现 `ordering_food_organization_published`
- `catalog-domain` 中不再存在名为 `Store` 的目录真相聚合

- [ ] **Step 5: 运行完整领域测试**

Run: `cd server && cargo test -p ordering-food-catalog-domain`

Expected: PASS，至少覆盖品牌目录、门店目录、定价、可售状态、展示规则、负价校验等核心约束。

- [ ] **Step 6: 仅当用户明确授权时，提交当前任务变更**

```bash
git add server/crates/catalog-domain
git commit -m "refactor: redraw catalog domain semantics"
```

## Task 3: 迁移应用层到 Catalog 用语，并显式接入 `Organization` scope facts

**Files:**
- Modify: `server/crates/catalog-application/Cargo.toml`
- Modify: `server/crates/catalog-application/src/lib.rs`
- Create: `server/crates/catalog-application/src/error.rs`
- Create: `server/crates/catalog-application/src/dto.rs`
- Create: `server/crates/catalog-application/src/module.rs`
- Create: `server/crates/catalog-application/src/ports.rs`
- Create: `server/crates/catalog-application/src/use_cases/mod.rs`
- Create: `server/crates/catalog-application/src/use_cases/bootstrap_brand_catalog.rs`
- Create: `server/crates/catalog-application/src/use_cases/attach_store_catalog.rs`
- Create: `server/crates/catalog-application/src/use_cases/create_category.rs`
- Create: `server/crates/catalog-application/src/use_cases/create_item.rs`
- Create: `server/crates/catalog-application/src/use_cases/upsert_store_item_listing.rs`
- Modify: `server/crates/catalog-application/tests/architecture.rs`
- Modify: `server/crates/catalog-published/src/lib.rs`
- Modify: `server/crates/catalog-integration/Cargo.toml`
- Modify: `server/crates/catalog-integration/src/lib.rs`
- Modify: `server/crates/organization-published/src/lib.rs`

- [ ] **Step 1: 先写失败用例，要求 Catalog 应用层通过 `Organization` published 事实附着 scope**

```rust
#[tokio::test]
async fn attach_store_catalog_rejects_unknown_store_scope() {
    let use_case = AttachStoreCatalog::new(
        Arc::new(FakeOrganizationScopeReader::missing()),
        Arc::new(FakeStoreCatalogRepository::default()),
        Arc::new(FakeTransactionManager::default()),
        Arc::new(FakeClock::fixed()),
        Arc::new(FakeIdGenerator::default()),
    );

    let error = use_case.execute(AttachStoreCatalogInput {
        brand_id: "brand-1".to_string(),
        store_id: "store-1".to_string(),
    }).await.unwrap_err();

    assert!(matches!(error, ApplicationError::NotFound { .. }));
}
```

- [ ] **Step 2: 运行应用层测试，确认端口与用例都还不存在**

Run: `cd server && cargo test -p ordering-food-catalog-application`

Expected: FAIL，主要缺失 `OrganizationScopeReader`、`AttachStoreCatalog`、`BootstrapBrandCatalog`、`UpsertStoreItemListing` 这些新语义对象。

- [ ] **Step 3: 实现 Catalog 应用层，用 `BrandCatalog` / `StoreCatalog` / `StoreItemListing` 替换旧 `MenuModule` 结构**

```rust
pub trait OrganizationScopeReader: Send + Sync {
    async fn get_brand(&self, brand_id: &str) -> Result<Option<BrandRef>, ApplicationError>;
    async fn get_store_scope(
        &self,
        brand_id: &str,
        store_id: &str,
    ) -> Result<Option<StoreSummary>, ApplicationError>;
}
```

- [ ] **Step 4: 升级 `catalog-published` 与 `organization-published`，把对外语言从占位改成真实稳定契约**

```rust
pub struct CatalogPriceFact {
    pub item_id: String,
    pub store_id: String,
    pub price_amount: i64,
}

pub struct StoreSummary {
    pub brand_id: String,
    pub store_id: String,
    pub slug: String,
    pub name: String,
    pub currency_code: String,
    pub timezone: String,
    pub status: String,
}
```

- [ ] **Step 5: 只在 `catalog-integration` 中补命名明确的 adapter / translator 占位，不实现完整网络**

Run: `cd server && cargo test -p ordering-food-catalog-application --test architecture && cargo test -p ordering-food-catalog-integration && cargo test -p ordering-food-catalog-published && cargo test -p ordering-food-organization-published`

Expected: PASS，并且 architecture test 至少保证：
- `catalog-application` 可以依赖 `ordering_food_organization_published`
- `catalog-application` 不能依赖未来的 `organization-domain` 或 `organization-application`
- `catalog-integration` 里不再保留 `ExternalFactTranslator` 这种泛化命名

- [ ] **Step 6: 再跑一次应用层主测试**

Run: `cd server && cargo test -p ordering-food-catalog-application`

Expected: PASS，至少覆盖品牌目录初始化、门店目录附着、分类创建、菜品创建、定价 / 可售 / 展示规则更新。

- [ ] **Step 7: 仅当用户明确授权时，提交当前任务变更**

```bash
git add server/crates/catalog-application \
  server/crates/catalog-published/src/lib.rs \
  server/crates/catalog-integration/Cargo.toml \
  server/crates/catalog-integration/src/lib.rs \
  server/crates/organization-published/src/lib.rs
git commit -m "refactor: move catalog application to organization scope facts"
```

## Task 4: 建立 `catalog` schema 与 SQLx 持久化，并完成从 `menu.*` 的数据回填

**Files:**
- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050301_catalog_context.up.sql`
- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050301_catalog_context.down.sql`
- Modify: `server/crates/database-infrastructure-sqlx/src/bin/migration-info.rs`
- Modify: `server/crates/catalog-infrastructure-sqlx/Cargo.toml`
- Modify: `server/crates/catalog-infrastructure-sqlx/src/lib.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/module.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/brand_catalog_repository.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/store_catalog_repository.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/category_repository.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/item_repository.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/store_item_listing_repository.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/brand_catalog_read_repository.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/store_catalog_read_repository.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/category_read_repository.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/item_read_repository.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/src/transaction.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/tests/architecture.rs`
- Create: `server/crates/catalog-infrastructure-sqlx/tests/repositories.rs`

- [ ] **Step 1: 先写失败的 repository / schema ownership 测试**

```rust
#[sqlx::test(migrator = "MIGRATOR")]
async fn catalog_store_item_listing_persists_price_and_sellable_status(pool: PgPool) {
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM information_schema.tables WHERE table_schema = 'catalog' AND table_name = 'store_item_listings'"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(count, 1);
}
```

- [ ] **Step 2: 运行持久化测试，确认 `catalog` schema 和仓储都还不存在**

Run: `cd server && cargo test -p ordering-food-catalog-infrastructure-sqlx --test repositories --test architecture && cargo test -p ordering-food-database-infrastructure-sqlx --bin migration-info`

Expected: FAIL，因为 package 中还没有目标仓储文件，也没有 `catalog` schema migration，且 `migration-info` 还没有登记 `202604050301_catalog_context`。

- [ ] **Step 3: 新建 `catalog` schema 与数据回填 migration，不要改写基线 migration**

```sql
CREATE SCHEMA IF NOT EXISTS catalog;

CREATE TABLE catalog.brand_catalogs (...);
CREATE TABLE catalog.store_catalogs (...);
CREATE TABLE catalog.categories (...);
CREATE TABLE catalog.items (...);
CREATE TABLE catalog.store_item_listings (...);

INSERT INTO catalog.store_catalogs (...)
SELECT ...
FROM menu.stores;
```

- [ ] **Step 4: 实现 SQLx 仓储与模块装配，明确 `BrandCatalog` / `StoreCatalog` / `StoreItemListing` 的持久化边界**

Run: `cd server && cargo test -p ordering-food-catalog-infrastructure-sqlx --test repositories`

Expected: PASS，至少覆盖：
- 旧 `menu.items.price_amount` 被回填为新的 store-level listing 定价
- 分类和菜品从旧表迁移到品牌目录语义
- store-level listing 上的可售状态与展示规则可读可写
- 仓储错误信息已经全部改成 `catalog` 语义

- [ ] **Step 5: 增加 architecture guard，禁止新持久化代码继续直接读写 `menu.*`**

Run: `cd server && cargo test -p ordering-food-catalog-infrastructure-sqlx --test architecture`

Expected: PASS，且规则至少覆盖：
- `server/crates/catalog-infrastructure-sqlx/src/*.rs` 中不允许出现 `FROM menu.`、`INSERT INTO menu.`、`UPDATE menu.`
- 允许 `menu.*` 只出现在新增 migration 文件的回填 SQL 中

- [ ] **Step 6: 跑一次 Catalog 持久化全量测试**

Run: `cd server && cargo test -p ordering-food-catalog-infrastructure-sqlx --tests && cargo test -p ordering-food-database-infrastructure-sqlx --bin migration-info`

Expected: PASS。

- [ ] **Step 7: 仅当用户明确授权时，提交当前任务变更**

```bash
git add server/crates/database-infrastructure-sqlx/migrations/202604050301_catalog_context.up.sql \
  server/crates/database-infrastructure-sqlx/migrations/202604050301_catalog_context.down.sql \
  server/crates/database-infrastructure-sqlx/src/bin/migration-info.rs \
  server/crates/catalog-infrastructure-sqlx
git commit -m "refactor: migrate catalog persistence to dedicated schema"
```

## Task 5: 切换 API 组合层与 HTTP 合约到 `Catalog`

**Files:**
- Modify: `server/apps/api/Cargo.toml`
- Modify: `server/apps/api/src/config.rs`
- Create: `server/apps/api/src/composition/contexts/catalog.rs`
- Modify: `server/apps/api/src/composition/contexts/mod.rs`
- Modify: `server/apps/api/src/composition/registry.rs`
- Create: `server/apps/api/src/routes/catalog.rs`
- Modify: `server/apps/api/src/routes/mod.rs`
- Modify: `server/apps/api/src/ts_bindings.rs`
- Create: `server/apps/api/tests/catalog_architecture.rs`
- Modify: `server/apps/api/tests/architecture.rs`
- Modify: `server/apps/api/tests/order_architecture.rs`
- Modify: `server/apps/api/tests/platform_architecture.rs`

- [ ] **Step 1: 先写一个失败的 API 架构测试，要求应用壳层已经切到 `catalog` 命名**

```rust
#[test]
fn catalog_routes_use_catalog_prefix() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/routes/catalog.rs"),
    )
    .unwrap();

    assert!(source.contains("pub(crate) const CATALOG_ROUTE_PREFIX: &str = \"/api/catalog\";"));
    assert!(!source.contains("/api/menu"));
}
```

- [ ] **Step 2: 运行 API 侧新测试，确认文件和前缀尚未切换**

Run: `cd server && cargo test -p ordering-food-api --test catalog_architecture`

Expected: FAIL，因为 `src/routes/catalog.rs` 和 `src/composition/contexts/catalog.rs` 还不存在。

- [ ] **Step 3: 用 `CatalogModule` 重写组合层，并把当前 seed 逻辑改成“品牌目录 + 门店目录 + store item listing”**

```rust
pub fn register_catalog() -> ApiContextRegistration {
    let descriptor = ContextDescriptor {
        id: "catalog",
        depends_on: &[],
    };

    ApiContextRegistration::without_migration(descriptor, catalog_bootstrap_registration)
}
```

- [ ] **Step 4: 把 HTTP 契约与 TS 导出从 `Menu*` 切到 `Catalog*`，并改用 `/api/catalog`**

Run: `cd server && cargo test -p ordering-food-api --lib routes::catalog::tests && cargo test -p ordering-food-api --lib composition::contexts::catalog::tests && cargo test -p ordering-food-api --lib ts_bindings::tests::export_bindings_writes_expected_contract_files`

Expected: PASS，并满足以下约束：
- 不再导出 `MenuStoreResponse`、`MenuCategoryResponse`、`MenuItemResponse`
- 新导出的契约至少包含 `CatalogStoreCatalogResponse`、`CatalogCategoryResponse`、`CatalogItemResponse`
- `apps/api` 不再依赖 `ordering-food-menu-*`

- [ ] **Step 5: 只把配置保留为单品牌本地默认值，不在 Catalog 内部制造第二个品牌真相源**

Run: `cd server && cargo test -p ordering-food-api config::tests --lib`

Expected: PASS，配置测试至少覆盖本地默认 brand scope 值和环境变量 override，并明确不会覆盖已存在的 `Organization` brand/store identity。

- [ ] **Step 6: 跑 API 侧全部相关测试**

Run: `cd server && cargo test -p ordering-food-api --lib && cargo test -p ordering-food-api --test architecture && cargo test -p ordering-food-api --test order_architecture && cargo test -p ordering-food-api --test platform_architecture && cargo test -p ordering-food-api --test catalog_architecture`

Expected: PASS，且 `server/apps/api/src/routes/menu.rs` 与 `server/apps/api/src/composition/contexts/menu.rs` 已经不再被编译链路引用。

- [ ] **Step 7: 仅当用户明确授权时，提交当前任务变更**

```bash
git add server/apps/api/Cargo.toml \
  server/apps/api/src/config.rs \
  server/apps/api/src/composition/contexts/catalog.rs \
  server/apps/api/src/composition/contexts/mod.rs \
  server/apps/api/src/composition/registry.rs \
  server/apps/api/src/routes/catalog.rs \
  server/apps/api/src/routes/mod.rs \
  server/apps/api/src/ts_bindings.rs \
  server/apps/api/tests/catalog_architecture.rs \
  server/apps/api/tests/architecture.rs \
  server/apps/api/tests/order_architecture.rs \
  server/apps/api/tests/platform_architecture.rs
git commit -m "refactor: switch api shell from menu to catalog"
```

## Task 6: 删除旧 `menu` crate 与 schema 遗留，并收紧最终守卫

**Files:**
- Modify: `server/Cargo.toml`
- Modify: `server/apps/api/tests/context_skeleton_architecture.rs`
- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050302_drop_menu_context.up.sql`
- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050302_drop_menu_context.down.sql`
- Modify: `server/crates/database-infrastructure-sqlx/src/bin/migration-info.rs`
- Delete: `server/crates/menu-domain/Cargo.toml`
- Delete: `server/crates/menu-domain/src/category.rs`
- Delete: `server/crates/menu-domain/src/category_id.rs`
- Delete: `server/crates/menu-domain/src/error.rs`
- Delete: `server/crates/menu-domain/src/item.rs`
- Delete: `server/crates/menu-domain/src/item_id.rs`
- Delete: `server/crates/menu-domain/src/lib.rs`
- Delete: `server/crates/menu-domain/src/status.rs`
- Delete: `server/crates/menu-domain/src/store.rs`
- Delete: `server/crates/menu-domain/src/store_id.rs`
- Delete: `server/crates/menu-application/Cargo.toml`
- Delete: `server/crates/menu-application/src/dto.rs`
- Delete: `server/crates/menu-application/src/error.rs`
- Delete: `server/crates/menu-application/src/lib.rs`
- Delete: `server/crates/menu-application/src/module.rs`
- Delete: `server/crates/menu-application/src/ports.rs`
- Delete: `server/crates/menu-application/src/use_cases/create_category.rs`
- Delete: `server/crates/menu-application/src/use_cases/create_item.rs`
- Delete: `server/crates/menu-application/src/use_cases/create_store.rs`
- Delete: `server/crates/menu-application/src/use_cases/mod.rs`
- Delete: `server/crates/menu-infrastructure-sqlx/Cargo.toml`
- Delete: `server/crates/menu-infrastructure-sqlx/src/category_read_repository.rs`
- Delete: `server/crates/menu-infrastructure-sqlx/src/category_repository.rs`
- Delete: `server/crates/menu-infrastructure-sqlx/src/item_read_repository.rs`
- Delete: `server/crates/menu-infrastructure-sqlx/src/item_repository.rs`
- Delete: `server/crates/menu-infrastructure-sqlx/src/lib.rs`
- Delete: `server/crates/menu-infrastructure-sqlx/src/module.rs`
- Delete: `server/crates/menu-infrastructure-sqlx/src/store_read_repository.rs`
- Delete: `server/crates/menu-infrastructure-sqlx/src/store_repository.rs`
- Delete: `server/crates/menu-infrastructure-sqlx/src/transaction.rs`
- Delete: `server/crates/menu-infrastructure-sqlx/tests/repositories.rs`
- Delete: `server/apps/api/src/composition/contexts/menu.rs`
- Delete: `server/apps/api/src/routes/menu.rs`
- Delete: `server/apps/api/tests/menu_architecture.rs`

- [ ] **Step 1: 先让 context skeleton 守卫明确拒绝旧 `menu-*` 业务 crate**

```rust
#[test]
fn workspace_members_no_longer_include_menu_business_crates() {
    let manifest = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml"),
    )
    .unwrap();

    for member in [
        "crates/menu-domain",
        "crates/menu-application",
        "crates/menu-infrastructure-sqlx",
    ] {
        assert!(!manifest.contains(member), "unexpected workspace member: {member}");
    }
}
```

- [ ] **Step 2: 删除旧 `menu` crate 源码和 API 文件，并从 workspace 中彻底移除成员**

Run: `cd server && cargo test -p ordering-food-api --test context_skeleton_architecture`

Expected: FAIL 在删除前出现，删除并更新 workspace 后转为 PASS。

- [ ] **Step 3: 添加最终数据库清理 migration，删除旧 `menu` schema**

Run: `cd server && cargo test -p ordering-food-catalog-infrastructure-sqlx --tests && cargo test -p ordering-food-database-infrastructure-sqlx --bin migration-info`

Expected: PASS，且所有 SQLx 测试都仅依赖 `catalog` schema；`menu` schema 只在 migration 顺序中作为历史回填来源存在，最终态不再被查询。

- [ ] **Step 4: 用 grep 做一次术语残留扫描，只允许明确白名单存在**

Run: `cd server && rg -n "ordering_food_menu|MenuModule|MenuStatus|/api/menu|\\bmenu\\." apps/api crates/catalog-* crates/organization-published Cargo.toml`

Expected: no matches。

Run: `cd server && rg -n "menu_item_id" crates/order-domain`

Expected: 仅命中 `server/crates/order-domain/src/menu_item_id.rs` 及其测试引用，因为该遗留命名属于后续 `Ordering` 计划范围。

- [ ] **Step 5: 跑 Phase 2C 最终验证命令**

Run: `cd server && cargo test -p ordering-food-api --tests && cargo test -p ordering-food-catalog-domain && cargo test -p ordering-food-catalog-application && cargo test -p ordering-food-catalog-infrastructure-sqlx --tests && cargo test -p ordering-food-catalog-published && cargo test -p ordering-food-catalog-integration && cargo test -p ordering-food-organization-published && cargo clippy -p ordering-food-api -p ordering-food-catalog-domain -p ordering-food-catalog-application -p ordering-food-catalog-infrastructure-sqlx --tests -- -D warnings`

Expected: PASS。

- [ ] **Step 6: 仅当用户明确授权时，提交当前任务变更**

```bash
git add server/Cargo.toml \
  server/apps/api/tests/context_skeleton_architecture.rs \
  server/crates/database-infrastructure-sqlx/migrations/202604050302_drop_menu_context.up.sql \
  server/crates/database-infrastructure-sqlx/migrations/202604050302_drop_menu_context.down.sql \
  server/crates/database-infrastructure-sqlx/src/bin/migration-info.rs
git add -u server/crates/menu-domain \
  server/crates/menu-application \
  server/crates/menu-infrastructure-sqlx \
  server/apps/api/src/composition/contexts/menu.rs \
  server/apps/api/src/routes/menu.rs \
  server/apps/api/tests/menu_architecture.rs
git commit -m "refactor: remove legacy menu context after catalog cutover"
```

## Execution Notes

- 领域建模上，`BrandCatalog` 负责品牌级目录骨架，`StoreCatalog` 负责门店级可售范围，`StoreItemListing` 负责 store-specific 的价格、可售状态和展示规则；不要再把 `Store` 当成 `Catalog` 自己拥有的组织聚合。
- API 行为上，优先保留现有读能力的业务语义与响应内容，但不保留 `/api/menu` 路径兼容；如果执行时发现前端强依赖旧路径，请先停下来拆分出单独的兼容计划，不要把 `menu` 与 `catalog` 混在同一条主线里长期共存。
- 数据迁移上，先通过新增 migration 把数据搬到 `catalog`，验证通过后再 drop `menu` schema；不要在同一步里同时做结构重画、路由切换和旧 schema 删除。
- 边界上，`Organization` 只提供 scope facts，`Catalog` 不能把 `brand` / `store` 的生命周期管理、组织命名真相、成员关系判断重新长回自己体内。

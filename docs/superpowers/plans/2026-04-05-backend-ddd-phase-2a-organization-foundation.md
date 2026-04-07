# Backend DDD Phase 2A Organization Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将当前由 `menu` 持有的门店与品牌真相抽离为 `Organization` 上下文基础，实现稳定的品牌/门店写模型、读模型与 `published scope facts`，并在不提前展开 `Catalog` 迁移的前提下，为后续 `Catalog` / `Access` / `Ordering` 提供可依赖的作用域事实源。

**Architecture:** 本阶段采用“先立真相所有权，再保留兼容读桥”的非破坏性迁移策略。`Organization` 将新增自己的 `domain` / `application` / `published` / `infrastructure-sqlx` 组合、数据库 schema 与应用装配；原 `menu.stores` 从写真相退化为兼容视图，仅用于当前 `menu` 的存量读路径与分类/菜品校验，直到后续 `Catalog` 迁移完成后再移除。`Catalog` / `Access` / `Ordering` 在本阶段不直接迁移，只依赖本阶段建立的 `organization-published` 契约与稳定的 schema ownership。

**Tech Stack:** Rust 2024、Cargo workspace、Tokio、Axum、SQLx、PostgreSQL schema migration、现有 bootstrap registry、架构守卫测试、`cargo test` / `cargo nextest`

---

## Planned File Map

### 现有文件修改

- `server/Cargo.toml`
  将 `organization-infrastructure-sqlx` 注册为新的 workspace member。
- `server/apps/api/Cargo.toml`
  为 `apps/api` 增加 `ordering-food-organization-application` 与 `ordering-food-organization-infrastructure-sqlx` 依赖。
- `server/apps/api/src/composition/contexts/mod.rs`
  注册 `organization` 上下文，并让 `menu` 装配顺序落在 `organization` 之后。
- `server/apps/api/src/composition/contexts/menu.rs`
  移除 `CreateStoreInput` 与门店写入种子逻辑；改为仅依赖已存在的组织门店作用域来补种分类与菜品。
- `server/apps/api/src/routes/menu.rs`
  适配 `MenuModule` 构造签名变化与新的门店作用域只读接口；保持 HTTP 响应不变。
- `server/apps/api/tests/context_skeleton_architecture.rs`
  扩展工作区守卫，覆盖 `organization-infrastructure-sqlx` 成员存在。
- `server/apps/api/tests/menu_architecture.rs`
  增加守卫，禁止 `menu` 继续暴露 `CreateStore` 或重新持有 `Store` 聚合。
- `server/crates/menu-domain/Cargo.toml`
  删除与门店聚合相关的依赖残留，仅保留分类/菜品所需依赖。
- `server/crates/menu-domain/src/lib.rs`
  移除 `store` 模块导出，只保留 `StoreId` 作为分类/菜品的外部作用域标识值对象。
- `server/crates/menu-application/Cargo.toml`
  对齐新的门店作用域只读端口依赖。
- `server/crates/menu-application/src/lib.rs`
  移除 `CreateStore` 导出，改为导出新的门店作用域查询端口。
- `server/crates/menu-application/src/dto.rs`
  将当前门店读模型显式降级为“组织门店作用域快照”的应用 DTO，而不是 `menu` 自己的聚合真相。
- `server/crates/menu-application/src/module.rs`
  删除 `create_store` 组装，改为注入门店作用域只读仓储。
- `server/crates/menu-application/src/ports.rs`
  用只读 `StoreScopeRepository` / `StoreScopeReadRepository` 替换当前写导向的 `StoreRepository`。
- `server/crates/menu-application/src/use_cases/create_category.rs`
  校验门店存在性时改为依赖组织作用域快照，而不是 `menu::Store` 聚合。
- `server/crates/menu-application/src/use_cases/create_item.rs`
  同步改为依赖组织作用域快照。
- `server/crates/menu-application/src/use_cases/mod.rs`
  删除 `create_store` 导出，保留分类/菜品用例。
- `server/crates/menu-infrastructure-sqlx/Cargo.toml`
  对齐新的只读端口与测试依赖。
- `server/crates/menu-infrastructure-sqlx/src/lib.rs`
  移除 `store_repository` 导出，保留读路径适配器。
- `server/crates/menu-infrastructure-sqlx/src/module.rs`
  更新 `MenuModule` 组装参数，不再创建门店写仓储。
- `server/crates/menu-infrastructure-sqlx/src/store_read_repository.rs`
  从兼容视图 `menu.stores` 读取门店作用域快照，并补充按 `store_id` 查询能力。
- `server/crates/menu-infrastructure-sqlx/tests/repositories.rs`
  增加兼容视图读取与只读行为测试，移除门店写仓储相关断言。
- `server/crates/organization-domain/Cargo.toml`
  增加领域实现所需依赖。
- `server/crates/organization-domain/src/lib.rs`
  导出品牌、门店、状态、错误与标识类型。
- `server/crates/organization-application/Cargo.toml`
  增加应用层依赖，包括 `organization-published` 与 `platform-kernel`。
- `server/crates/organization-application/src/lib.rs`
  导出组织模块、DTO、端口、查询服务与用例。
- `server/crates/organization-published/Cargo.toml`
  增加 published 契约实现所需依赖。
- `server/crates/organization-published/src/lib.rs`
  导出品牌/门店引用、摘要与状态变化契约。
### 新文件创建

- `server/apps/api/src/composition/contexts/organization.rs`
- `server/apps/api/tests/organization_architecture.rs`

- `server/crates/organization-domain/src/brand.rs`
- `server/crates/organization-domain/src/brand_id.rs`
- `server/crates/organization-domain/src/error.rs`
- `server/crates/organization-domain/src/status.rs`
- `server/crates/organization-domain/src/store.rs`
- `server/crates/organization-domain/src/store_id.rs`
- `server/crates/organization-domain/tests/organization_model.rs`

- `server/crates/organization-application/src/dto.rs`
- `server/crates/organization-application/src/module.rs`
- `server/crates/organization-application/src/ports.rs`
- `server/crates/organization-application/src/use_cases/create_brand.rs`
- `server/crates/organization-application/src/use_cases/create_store.rs`
- `server/crates/organization-application/src/use_cases/mod.rs`
- `server/crates/organization-application/tests/create_store.rs`

- `server/crates/organization-published/src/events.rs`
- `server/crates/organization-published/src/refs.rs`
- `server/crates/organization-published/src/store_scope.rs`
- `server/crates/organization-published/tests/contracts.rs`

- `server/crates/organization-infrastructure-sqlx/Cargo.toml`
- `server/crates/organization-infrastructure-sqlx/src/lib.rs`
- `server/crates/organization-infrastructure-sqlx/src/module.rs`
- `server/crates/organization-infrastructure-sqlx/src/brand_repository.rs`
- `server/crates/organization-infrastructure-sqlx/src/store_read_repository.rs`
- `server/crates/organization-infrastructure-sqlx/src/store_repository.rs`
- `server/crates/organization-infrastructure-sqlx/src/transaction.rs`
- `server/crates/organization-infrastructure-sqlx/tests/repositories.rs`

- `server/crates/database-infrastructure-sqlx/migrations/202604050101_organization_foundation.up.sql`
- `server/crates/database-infrastructure-sqlx/migrations/202604050101_organization_foundation.down.sql`

### 现有文件删除

- `server/crates/menu-domain/src/store.rs`
- `server/crates/menu-application/src/use_cases/create_store.rs`
- `server/crates/menu-infrastructure-sqlx/src/store_repository.rs`

## 范围边界

本阶段只做 `Organization` 基础抽取与 `published scope facts` 建立，明确不做以下事项：

- 不做 `menu -> catalog` 命名迁移与分类/菜品边界重画。
- 不做 `authz -> access` 重命名或成员关系模型迁移。
- 不做 `order -> ordering + fulfillment` 拆分。
- 不做 outbox、dispatcher、projector 网络。
- 不引入跨上下文同步白名单机制的长期实现，只保留为后续阶段准备的 published 契约。

## 关键判断与取舍

### 取舍 1：本阶段采用“组织真相表 + `menu.stores` 兼容视图”，而不是立即把 `menu` 全量迁到 `Catalog`

选择该方案的原因：

- 它把 Phase 2A 的目标限制在“真相所有权转移”与“published scope facts 建立”，不会提前把分类/菜品迁移混入。
- 它能保留当前 `/api/menu/store`、分类创建、菜品创建的外部行为，不需要在同一阶段里把所有 `menu` 调用点重写为新上下文。
- 它允许后续 `Catalog` 计划直接删除兼容视图，而不是再从 `menu` 内部抽一次门店真相。

不选择“继续让 `menu` 写 `menu.stores`，Organization 只做镜像”的原因：

- 这会让真相所有权仍然停留在 `menu`，与本阶段目标冲突。
- 后续 `Access` / `Ordering` 无法把 `organization-published` 视为稳定事实源。

不选择“本阶段立刻让 `menu` 通过 integration read model 完整消费 `Organization`”的原因：

- 现阶段尚未建立 Phase 3 的 outbox / projector 主干。
- 这样会把 Catalog 迁移的一半提前塞进本阶段，导致任务失焦。

### 取舍 2：本阶段建立 `Brand`，但只支持默认单品牌引导

选择该方案的原因：

- 上位设计已经明确品牌是长期稳定的一等概念，不能继续被门店模型吞掉。
- 当前运行形态仍是单品牌，因此只需要支持“默认品牌 + 门店归属”的最小闭环。

不选择“先只建 `Store`，品牌以后再补”的原因：

- 这会让后续 `Access` / `Catalog` 再次经历外键和 published 契约升级，增加二次迁移成本。

## 与后续计划的依赖关系

- 前置依赖：本计划只能在 [2026-04-05-backend-ddd-phase-1-boundary-skeleton.md](/Users/ggg/private/ordering-food/docs/superpowers/plans/2026-04-05-backend-ddd-phase-1-boundary-skeleton.md) 完成后执行，因为它依赖 Phase 1 已建立的 `organization-*` / `*-published` / `*-integration` 骨架与平台抽象。
- 对 `Catalog` 的关系：后续 `Catalog` 迁移必须依赖本阶段产出的 `organization.brands`、`organization.stores`、`organization-published` 作用域契约，以及 `menu.stores` 兼容视图。`Catalog` 计划负责删除兼容视图、移除 `menu` 残余命名，并把分类/菜品正式迁入 `catalog`。
- 对 `Access` 的关系：后续 `Access` 迁移只应消费本阶段产出的 `BrandRef`、`StoreRef`、`StoreSummary` 等 scope facts，不再把 `authz.store_memberships.store_id` 视为“无来源的裸 UUID”。`Access` 可以在本阶段完成后独立开始，不需要等待 `Catalog`。
- 对 `Ordering` 的关系：后续 `Ordering` 拆分至少依赖两类上游事实。
  第一类是本阶段产出的组织作用域事实。
  第二类是后续 `Catalog` 发布的商品与价格事实。
  因此 `Ordering` 计划必须晚于本阶段，并且完整业务迁移应晚于 `Catalog` 作用域/商品契约稳定之后。
- 推荐顺序：`Phase 1 -> Phase 2A Organization foundation -> Phase 2B Access + Identity purification -> Phase 2C Catalog migration -> Phase 2D Ordering / Fulfillment split`。其中 `Phase 2B` 与 `Phase 2C` 在依赖上都只要求 `Phase 2A` 完成；如果人力允许，可以并行准备，但实际 cutover 仍建议先做 `Phase 2B`，先把授权边界从旧 `authz` 语义中抽离，再进入目录与订单/履约的深层重画。

## Task 1：建立 Organization 领域模型与 published scope facts

**Files:**
- Modify: `server/crates/organization-domain/Cargo.toml`
- Modify: `server/crates/organization-domain/src/lib.rs`
- Create: `server/crates/organization-domain/src/brand.rs`
- Create: `server/crates/organization-domain/src/brand_id.rs`
- Create: `server/crates/organization-domain/src/error.rs`
- Create: `server/crates/organization-domain/src/status.rs`
- Create: `server/crates/organization-domain/src/store.rs`
- Create: `server/crates/organization-domain/src/store_id.rs`
- Create: `server/crates/organization-domain/tests/organization_model.rs`
- Modify: `server/crates/organization-published/Cargo.toml`
- Modify: `server/crates/organization-published/src/lib.rs`
- Create: `server/crates/organization-published/src/events.rs`
- Create: `server/crates/organization-published/src/refs.rs`
- Create: `server/crates/organization-published/src/store_scope.rs`
- Create: `server/crates/organization-published/tests/contracts.rs`

- [ ] **Step 1: 先写失败的领域测试与 published 契约测试**

```rust
// server/crates/organization-domain/tests/organization_model.rs
use ordering_food_organization_domain::{Brand, BrandId, OrganizationStatus, Store, StoreId};
use time::macros::datetime;

#[test]
fn store_belongs_to_brand_and_preserves_scope_fields() {
    let brand = Brand::create(
        BrandId::new("brand-1"),
        "ordering-food",
        "Ordering Food",
        OrganizationStatus::Active,
        datetime!(2026-04-05 08:00 UTC),
    )
    .unwrap();

    let store = Store::create(
        StoreId::new("store-1"),
        brand.id().clone(),
        "demo-kitchen",
        "Demo Kitchen",
        "CNY",
        "Asia/Shanghai",
        OrganizationStatus::Active,
        datetime!(2026-04-05 08:01 UTC),
    )
    .unwrap();

    assert_eq!(store.brand_id(), brand.id());
    assert_eq!(store.currency_code(), "CNY");
    assert_eq!(store.timezone(), "Asia/Shanghai");
}
```

```rust
// server/crates/organization-published/tests/contracts.rs
use ordering_food_organization_published::{BrandRef, StoreRef, StoreSummary};

#[test]
fn store_summary_contains_stable_scope_fields_for_other_contexts() {
    let summary = StoreSummary {
        store_id: "store-1".to_string(),
        brand_id: "brand-1".to_string(),
        slug: "demo-kitchen".to_string(),
        name: "Demo Kitchen".to_string(),
        currency_code: "CNY".to_string(),
        timezone: "Asia/Shanghai".to_string(),
        status: "active".to_string(),
    };

    let brand_ref = BrandRef {
        brand_id: summary.brand_id.clone(),
    };
    let store_ref = StoreRef {
        store_id: summary.store_id.clone(),
        brand_id: summary.brand_id.clone(),
    };

    assert_eq!(brand_ref.brand_id, "brand-1");
    assert_eq!(store_ref.store_id, "store-1");
}
```

- [ ] **Step 2: 运行测试确认当前骨架不足以通过**

Run: `cd server && cargo test -p ordering-food-organization-domain --test organization_model && cargo test -p ordering-food-organization-published --test contracts`

Expected: FAIL，报错应集中在 `Brand` / `Store` / `OrganizationStatus` / `StoreSummary` 等符号尚未实现或未导出。

- [ ] **Step 3: 实现最小领域模型与对外 published 契约**

```rust
// server/crates/organization-published/src/refs.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrandRef {
    pub brand_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreRef {
    pub store_id: String,
    pub brand_id: String,
}
```

```rust
// server/crates/organization-published/src/store_scope.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreSummary {
    pub store_id: String,
    pub brand_id: String,
    pub slug: String,
    pub name: String,
    pub currency_code: String,
    pub timezone: String,
    pub status: String,
}
```

```rust
// server/crates/organization-published/src/events.rs
use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreStatusChanged {
    pub store_id: String,
    pub brand_id: String,
    pub previous_status: String,
    pub current_status: String,
    pub occurred_at: Timestamp,
}
```

- [ ] **Step 4: 运行领域与契约测试**

Run: `cd server && cargo test -p ordering-food-organization-domain && cargo test -p ordering-food-organization-published`

Expected: PASS，`organization-domain` 的领域测试与既有 architecture test 全部通过；`organization-published` 契约测试通过。

- [ ] **Step 5: 仅当用户明确授权时提交本任务**

```bash
git add \
  server/crates/organization-domain/Cargo.toml \
  server/crates/organization-domain/src/lib.rs \
  server/crates/organization-domain/src/brand.rs \
  server/crates/organization-domain/src/brand_id.rs \
  server/crates/organization-domain/src/error.rs \
  server/crates/organization-domain/src/status.rs \
  server/crates/organization-domain/src/store.rs \
  server/crates/organization-domain/src/store_id.rs \
  server/crates/organization-domain/tests/organization_model.rs \
  server/crates/organization-published/Cargo.toml \
  server/crates/organization-published/src/lib.rs \
  server/crates/organization-published/src/events.rs \
  server/crates/organization-published/src/refs.rs \
  server/crates/organization-published/src/store_scope.rs \
  server/crates/organization-published/tests/contracts.rs
git commit -m "feat: add organization domain and published scope facts"
```

## Task 2：建立 Organization 应用层与模块装配接口

**Files:**
- Modify: `server/crates/organization-application/Cargo.toml`
- Modify: `server/crates/organization-application/src/lib.rs`
- Create: `server/crates/organization-application/src/dto.rs`
- Create: `server/crates/organization-application/src/module.rs`
- Create: `server/crates/organization-application/src/ports.rs`
- Create: `server/crates/organization-application/src/use_cases/create_brand.rs`
- Create: `server/crates/organization-application/src/use_cases/create_store.rs`
- Create: `server/crates/organization-application/src/use_cases/mod.rs`
- Create: `server/crates/organization-application/tests/create_store.rs`

- [ ] **Step 1: 先写失败的应用层测试，锁定品牌存在校验与 active store 查询行为**

```rust
// server/crates/organization-application/tests/create_store.rs
use ordering_food_organization_application::{
    ApplicationError, CreateStore, CreateStoreInput,
};
use std::sync::Arc;

#[tokio::test]
async fn create_store_requires_existing_brand() {
    let use_case = test_create_store_use_case_without_brands();

    let error = use_case
        .execute(CreateStoreInput {
            brand_id: "brand-missing".to_string(),
            slug: "demo-kitchen".to_string(),
            name: "Demo Kitchen".to_string(),
            currency_code: "CNY".to_string(),
            timezone: "Asia/Shanghai".to_string(),
            status: "active".to_string(),
        })
        .await
        .unwrap_err();

    assert!(
        matches!(error, ApplicationError::NotFound { ref message } if message == "brand was not found")
    );
}

fn test_create_store_use_case_without_brands() -> CreateStore {
    CreateStore::new(
        Arc::new(FakeBrandRepository::default()),
        Arc::new(FakeStoreRepository::default()),
        Arc::new(FakeTransactionManager::default()),
        Arc::new(FixedClock::default()),
        Arc::new(FixedIdGenerator),
    )
}
```

- [ ] **Step 2: 运行测试确认应用层尚未实现**

Run: `cd server && cargo test -p ordering-food-organization-application --test create_store`

Expected: FAIL，报错应集中在 `CreateBrand` / `CreateStore` / `OrganizationModule` / 端口定义尚未存在。

- [ ] **Step 3: 实现组织应用层最小闭环**

```rust
// server/crates/organization-application/src/ports.rs
#[async_trait]
pub trait BrandRepository: Send + Sync {
    async fn find_by_id(
        &self,
        tx: &mut dyn TransactionContext,
        brand_id: &BrandId,
    ) -> Result<Option<Brand>, ApplicationError>;

    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        brand: &Brand,
    ) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait StoreRepository: Send + Sync {
    async fn find_by_id(
        &self,
        tx: &mut dyn TransactionContext,
        store_id: &StoreId,
    ) -> Result<Option<Store>, ApplicationError>;

    async fn insert(
        &self,
        tx: &mut dyn TransactionContext,
        store: &Store,
    ) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait StoreReadRepository: Send + Sync {
    async fn get_active(&self) -> Result<Option<StoreSummary>, ApplicationError>;
}
```

```rust
// server/crates/organization-application/src/module.rs
#[derive(Clone)]
pub struct OrganizationModule {
    pub create_brand: Arc<CreateBrand>,
    pub create_store: Arc<CreateStore>,
    pub store_queries: Arc<StoreQueryService>,
}
```

- [ ] **Step 4: 运行应用层测试**

Run: `cd server && cargo test -p ordering-food-organization-application`

Expected: PASS，创建品牌、创建门店、查询 active store 的单测全部通过。

- [ ] **Step 5: 仅当用户明确授权时提交本任务**

```bash
git add \
  server/crates/organization-application/Cargo.toml \
  server/crates/organization-application/src/lib.rs \
  server/crates/organization-application/src/dto.rs \
  server/crates/organization-application/src/module.rs \
  server/crates/organization-application/src/ports.rs \
  server/crates/organization-application/src/use_cases/create_brand.rs \
  server/crates/organization-application/src/use_cases/create_store.rs \
  server/crates/organization-application/src/use_cases/mod.rs \
  server/crates/organization-application/tests/create_store.rs
git commit -m "feat: add organization application layer"
```

## Task 3：新增 Organization SQLx 持久化 crate 与 schema/backfill/兼容视图迁移

**Files:**
- Modify: `server/Cargo.toml`
- Create: `server/crates/organization-infrastructure-sqlx/Cargo.toml`
- Create: `server/crates/organization-infrastructure-sqlx/src/lib.rs`
- Create: `server/crates/organization-infrastructure-sqlx/src/module.rs`
- Create: `server/crates/organization-infrastructure-sqlx/src/brand_repository.rs`
- Create: `server/crates/organization-infrastructure-sqlx/src/store_read_repository.rs`
- Create: `server/crates/organization-infrastructure-sqlx/src/store_repository.rs`
- Create: `server/crates/organization-infrastructure-sqlx/src/transaction.rs`
- Create: `server/crates/organization-infrastructure-sqlx/tests/repositories.rs`
- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050101_organization_foundation.up.sql`
- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050101_organization_foundation.down.sql`
- Modify: `server/crates/database-infrastructure-sqlx/src/bin/migration-info.rs`

- [ ] **Step 1: 先写失败的持久化集成测试**

```rust
// server/crates/organization-infrastructure-sqlx/tests/repositories.rs
#[sqlx::test(migrator = "ordering_food_database_infrastructure_sqlx::MIGRATOR")]
async fn organization_migration_creates_schema_tables_and_menu_compat_view(pool: PgPool) {
    let schemas = sqlx::query_scalar::<_, String>(
        r#"
        SELECT schema_name
        FROM information_schema.schemata
        WHERE schema_name IN ('organization', 'menu')
        ORDER BY schema_name
        "#,
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(schemas, vec!["menu".to_string(), "organization".to_string()]);
}
```

- [ ] **Step 2: 运行测试确认 crate 与迁移尚未就绪**

Run: `cd server && cargo test -p ordering-food-organization-infrastructure-sqlx && cargo test -p ordering-food-database-infrastructure-sqlx --bin migration-info`

Expected: FAIL，初始失败点应为 package 不存在、测试中的 `organization` schema / `menu.stores` 兼容视图不存在，或 `migration-info` 尚未登记 `202604050101_organization_foundation`。

- [ ] **Step 3: 添加 crate、迁移脚本与最小 SQLx 适配器**

```sql
-- server/crates/database-infrastructure-sqlx/migrations/202604050101_organization_foundation.up.sql
CREATE SCHEMA IF NOT EXISTS organization;

CREATE TABLE organization.brands (
    id UUID PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'inactive')),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ NULL
);

CREATE TABLE organization.stores (
    id UUID PRIMARY KEY,
    brand_id UUID NOT NULL REFERENCES organization.brands(id),
    slug TEXT NOT NULL,
    name TEXT NOT NULL,
    currency_code CHAR(3) NOT NULL,
    timezone TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'inactive')),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ NULL,
    CONSTRAINT organization_stores_brand_slug_unique UNIQUE (brand_id, slug)
);

-- 迁移现有 menu 真相到 organization 真相；默认品牌只在当前单品牌阶段作为过渡数据存在
INSERT INTO organization.brands (
    id, slug, name, status, created_at, updated_at, deleted_at
)
VALUES (
    '00000000-0000-4000-8000-000000000001',
    'ordering-food',
    'Ordering Food',
    'active',
    NOW(),
    NOW(),
    NULL
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO organization.stores (
    id, brand_id, slug, name, currency_code, timezone, status, created_at, updated_at, deleted_at
)
SELECT
    id,
    '00000000-0000-4000-8000-000000000001',
    slug,
    name,
    currency_code,
    timezone,
    status,
    created_at,
    updated_at,
    deleted_at
FROM menu.stores
ON CONFLICT (id) DO NOTHING;

DROP TABLE menu.stores;

CREATE VIEW menu.stores AS
SELECT
    s.id,
    s.slug,
    s.name,
    s.currency_code,
    s.timezone,
    s.status,
    s.created_at,
    s.updated_at,
    s.deleted_at
FROM organization.stores s;
```

- [ ] **Step 4: 运行组织持久化与迁移测试**

Run: `cd server && cargo test -p ordering-food-organization-infrastructure-sqlx && cargo test -p ordering-food-database-infrastructure-sqlx && cargo test -p ordering-food-database-infrastructure-sqlx --bin migration-info`

Expected: PASS，至少覆盖以下断言。
Expected: `organization` schema、`brands` / `stores` 表存在。
Expected: `menu.stores` 已退化为只读兼容视图。
Expected: 组织仓储可以插入品牌与门店，并能查询 active store。

- [ ] **Step 5: 仅当用户明确授权时提交本任务**

```bash
git add \
  server/Cargo.toml \
  server/crates/organization-infrastructure-sqlx/Cargo.toml \
  server/crates/organization-infrastructure-sqlx/src/lib.rs \
  server/crates/organization-infrastructure-sqlx/src/module.rs \
  server/crates/organization-infrastructure-sqlx/src/brand_repository.rs \
  server/crates/organization-infrastructure-sqlx/src/store_read_repository.rs \
  server/crates/organization-infrastructure-sqlx/src/store_repository.rs \
  server/crates/organization-infrastructure-sqlx/src/transaction.rs \
  server/crates/organization-infrastructure-sqlx/tests/repositories.rs \
  server/crates/database-infrastructure-sqlx/migrations/202604050101_organization_foundation.up.sql \
  server/crates/database-infrastructure-sqlx/migrations/202604050101_organization_foundation.down.sql \
  server/crates/database-infrastructure-sqlx/src/bin/migration-info.rs
git commit -m "feat: add organization persistence and migration bridge"
```

## Task 4：把 Organization 上下文接入应用装配，并先于 menu 完成默认品牌/门店引导

**Files:**
- Modify: `server/apps/api/Cargo.toml`
- Modify: `server/apps/api/src/composition/contexts/mod.rs`
- Create: `server/apps/api/src/composition/contexts/organization.rs`
- Create: `server/apps/api/tests/organization_architecture.rs`

- [ ] **Step 1: 先写失败的装配守卫测试**

```rust
// server/apps/api/tests/organization_architecture.rs
use std::{fs, path::Path};

#[test]
fn api_registers_organization_before_menu() {
    let source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/composition/contexts/mod.rs"))
            .unwrap();

    assert!(source.contains("mod organization;"));
    assert!(source.contains("organization::register_organization()"));
}
```

- [ ] **Step 2: 运行测试确认当前应用壳尚未注册 organization**

Run: `cd server && cargo test -p ordering-food-api --test organization_architecture`

Expected: FAIL，因为 `organization` context 还未注册，且 `apps/api` 尚未依赖对应 crate。

- [ ] **Step 3: 实现 `organization` bootstrap 与默认品牌/门店 seed**

```rust
// server/apps/api/src/composition/contexts/organization.rs
pub fn register_organization() -> ApiContextRegistration {
    let descriptor = ContextDescriptor {
        id: "organization",
        depends_on: &["database"],
    };

    ApiContextRegistration::without_migration(descriptor, organization_bootstrap_registration)
}
```

```rust
// seed 逻辑核心约束
// 1. 如果 active store 已存在，直接跳过
// 2. 如果没有任何品牌，先创建默认品牌
// 3. 如果没有 active store，创建默认门店
// 4. 不挂路由，只保留私有 module 与 readiness check
```

- [ ] **Step 4: 运行 API 装配测试与组织 seed 单测**

Run: `cd server && cargo test -p ordering-food-api --test organization_architecture && cargo test -p ordering-food-api seed_organization_if_empty_creates_default_brand_and_store -- --exact && cargo test -p ordering-food-api seed_organization_if_empty_skips_when_active_store_exists -- --exact`

Expected: PASS，`organization` context 已注册，且 `seed_organization_if_empty_creates_default_brand_and_store` 与 `seed_organization_if_empty_skips_when_active_store_exists` 两个测试都通过。

- [ ] **Step 5: 仅当用户明确授权时提交本任务**

```bash
git add \
  server/apps/api/Cargo.toml \
  server/apps/api/src/composition/contexts/mod.rs \
  server/apps/api/src/composition/contexts/organization.rs \
  server/apps/api/tests/organization_architecture.rs
git commit -m "feat: wire organization context into api bootstrap"
```

## Task 5：移除 menu 的门店真相写模型，保留基于兼容视图的当前读行为

**Files:**
- Modify: `server/apps/api/src/composition/contexts/menu.rs`
- Modify: `server/apps/api/src/routes/menu.rs`
- Modify: `server/apps/api/tests/context_skeleton_architecture.rs`
- Modify: `server/apps/api/tests/menu_architecture.rs`
- Modify: `server/crates/menu-domain/Cargo.toml`
- Modify: `server/crates/menu-domain/src/lib.rs`
- Delete: `server/crates/menu-domain/src/store.rs`
- Modify: `server/crates/menu-application/Cargo.toml`
- Modify: `server/crates/menu-application/src/lib.rs`
- Modify: `server/crates/menu-application/src/dto.rs`
- Modify: `server/crates/menu-application/src/module.rs`
- Modify: `server/crates/menu-application/src/ports.rs`
- Modify: `server/crates/menu-application/src/use_cases/create_category.rs`
- Modify: `server/crates/menu-application/src/use_cases/create_item.rs`
- Modify: `server/crates/menu-application/src/use_cases/mod.rs`
- Delete: `server/crates/menu-application/src/use_cases/create_store.rs`
- Modify: `server/crates/menu-infrastructure-sqlx/Cargo.toml`
- Modify: `server/crates/menu-infrastructure-sqlx/src/lib.rs`
- Modify: `server/crates/menu-infrastructure-sqlx/src/module.rs`
- Modify: `server/crates/menu-infrastructure-sqlx/src/store_read_repository.rs`
- Delete: `server/crates/menu-infrastructure-sqlx/src/store_repository.rs`
- Modify: `server/crates/menu-infrastructure-sqlx/tests/repositories.rs`

- [ ] **Step 1: 先写失败的边界守卫测试，禁止 `menu` 再持有门店聚合与 `CreateStore`**

```rust
// server/apps/api/tests/menu_architecture.rs
use std::{fs, path::Path};

#[test]
fn menu_no_longer_exports_store_truth_or_create_store() {
    let menu_domain =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/menu-domain/src/lib.rs"))
            .unwrap();
    let menu_application =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/menu-application/src/lib.rs"))
            .unwrap();

    assert!(!menu_domain.contains("mod store;"));
    assert!(!menu_application.contains("CreateStore"));
}
```

- [ ] **Step 2: 运行测试确认当前 `menu` 仍然拥有门店真相**

Run: `cd server && cargo test -p ordering-food-api --test menu_architecture`

Expected: FAIL，因为当前 `menu-domain` 仍导出 `Store`，`menu-application` 仍导出 `CreateStore`。

- [ ] **Step 3: 把 `menu` 改造成只消费组织作用域快照的兼容上下文**

```rust
// server/crates/menu-application/src/ports.rs
#[async_trait]
pub trait StoreScopeRepository: Send + Sync {
    async fn find_by_id(
        &self,
        store_id: &StoreId,
    ) -> Result<Option<StoreReadModel>, ApplicationError>;
}

#[async_trait]
pub trait StoreScopeReadRepository: Send + Sync {
    async fn get_active(&self) -> Result<Option<StoreReadModel>, ApplicationError>;
}
```

```rust
// server/apps/api/src/composition/contexts/menu.rs
// 新约束：
// 1. menu seed 不再创建 store
// 2. menu seed 只在 active store 已存在时补种分类和菜品
// 3. 如果没有 active store，则返回错误并阻止静默再生长旧真相
```

```rust
// server/crates/menu-infrastructure-sqlx/src/store_read_repository.rs
// 继续从 menu.stores 读取，但此时 menu.stores 已是 organization.stores 的兼容视图
```

- [ ] **Step 4: 运行 `menu` 相关测试，确认行为保持但所有权已转移**

Run: `cd server && cargo test -p ordering-food-menu-application && cargo test -p ordering-food-menu-infrastructure-sqlx && cargo test -p ordering-food-api --test menu_architecture`

Expected: PASS。
Expected: `menu` 不再包含门店写用例与写仓储。
Expected: 现有 `/api/menu/store` 与分类/菜品读取测试仍通过。
Expected: `menu` 创建分类/菜品时只校验“组织门店作用域存在”，不再依赖 `menu::Store` 聚合。

- [ ] **Step 5: 仅当用户明确授权时提交本任务**

```bash
git add \
  server/apps/api/src/composition/contexts/menu.rs \
  server/apps/api/src/routes/menu.rs \
  server/apps/api/tests/context_skeleton_architecture.rs \
  server/apps/api/tests/menu_architecture.rs \
  server/crates/menu-domain/Cargo.toml \
  server/crates/menu-domain/src/lib.rs \
  server/crates/menu-application/Cargo.toml \
  server/crates/menu-application/src/lib.rs \
  server/crates/menu-application/src/dto.rs \
  server/crates/menu-application/src/module.rs \
  server/crates/menu-application/src/ports.rs \
  server/crates/menu-application/src/use_cases/create_category.rs \
  server/crates/menu-application/src/use_cases/create_item.rs \
  server/crates/menu-application/src/use_cases/mod.rs \
  server/crates/menu-infrastructure-sqlx/Cargo.toml \
  server/crates/menu-infrastructure-sqlx/src/lib.rs \
  server/crates/menu-infrastructure-sqlx/src/module.rs \
  server/crates/menu-infrastructure-sqlx/src/store_read_repository.rs \
  server/crates/menu-infrastructure-sqlx/tests/repositories.rs
git rm \
  server/crates/menu-domain/src/store.rs \
  server/crates/menu-application/src/use_cases/create_store.rs \
  server/crates/menu-infrastructure-sqlx/src/store_repository.rs
git commit -m "refactor: move store truth out of menu into organization"
```

## 完成校验

- [ ] **Step 1: 运行组织与菜单相关测试矩阵**

Run: `cd server && cargo test -p ordering-food-organization-domain && cargo test -p ordering-food-organization-published && cargo test -p ordering-food-organization-application && cargo test -p ordering-food-organization-infrastructure-sqlx && cargo test -p ordering-food-menu-application && cargo test -p ordering-food-menu-infrastructure-sqlx && cargo test -p ordering-food-api --test organization_architecture && cargo test -p ordering-food-api --test menu_architecture && cargo test -p ordering-food-api --test context_skeleton_architecture`

Expected: PASS，且失败信息中不再出现 `CreateStore`、`menu::Store`、缺失 `organization` context、缺失 `organization-infrastructure-sqlx` workspace member 等旧边界迹象。

- [ ] **Step 2: 运行工作区回归验证**

Run: `cd server && cargo test`

Expected: PASS；若执行时间过长，可退而执行 `cd server && cargo nextest run`，预期同样 PASS。

- [ ] **Step 3: 仅当用户明确授权时提交整阶段收口 commit**

```bash
git add server
git commit -m "refactor: establish organization foundation context"
```

## 实施后的状态定义

达到本计划完成态时，应满足以下判断标准：

- `Organization` 拥有品牌与门店的唯一写真相。
- `organization-published` 已提供后续 `Catalog` / `Access` / `Ordering` 可直接依赖的作用域契约。
- `menu` 不再拥有 `Store` 聚合与 `CreateStore` 用例。
- `menu.stores` 只作为兼容视图存在，并在后续 `Catalog` 计划中被删除。
- `apps/api` 已能在启动时先建立默认品牌/门店，再补种现有菜单数据。
- 后续计划可以把 `Catalog`、`Access`、`Ordering` 的作用域依赖统一指向 `Organization`，而不是继续从 `menu` 或裸 UUID 推断。

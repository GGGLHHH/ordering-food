# Backend DDD Phase 2B: Access + Identity Purification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在不引入 event bus / outbox、不混入 `menu -> catalog` 或 `order -> ordering + fulfillment` 拆分的前提下，把现有 `authz` 语义完整迁移到 `Access`，并把 `Identity` 的对外边界压回“认证主体本位”：只保留账户、凭证、登录方式、会话与 token 生命周期职责，对外通过稳定的 `SubjectRef` 发布认证主体事实。

**Architecture:** 本阶段采用“先边界纯化，再事件化”的过渡策略。`Access` 先通过 `identity-published` 与前置 `Phase 2A` 已落地的 `organization-published` 建立稳定消费边界，具体同步适配器暂时放在 `apps/api` 组合根内显式装配，不在本阶段引入 projector、outbox 或异步传播主干。数据库层不修改已发布的 `authz` 历史迁移，而是新增一条 additive `access` 迁移，把旧数据向新 schema 复制并完成 crate 与命名切换。

**Tech Stack:** Rust 2024, Cargo workspace, Tokio, Axum, SQLx, Redis/JWT（Identity 运行时既有实现）, GitNexus 架构影响分析, `cargo test`, `cargo clippy`.

---

## Planned File Map

### Existing files to modify

- `server/Cargo.toml`
  从 workspace 中移除 `authz-*` members，加入 `access-infrastructure-sqlx` member。
- `server/apps/api/Cargo.toml`
  用 `ordering-food-access-*` 依赖替换 `ordering-food-authz-*` 依赖，并补齐 Phase 2B 所需的组合根适配依赖。
- `server/apps/api/src/http.rs`
  将当前认证提取语义从 `AuthenticatedUser` 收敛为 `AuthenticatedSubject`，把 HTTP 层外部语义对齐到认证主体。
- `server/apps/api/src/routes/auth.rs`
  让 `/api/auth/me` 等读路径从认证主体进入，再映射回 Identity 内部账户查询。
- `server/apps/api/src/routes/orders.rs`
  把订单履约权限校验从 `AuthorizationService` 切换到新的 `AccessService`，并更新测试替身。
- `server/apps/api/src/composition/contexts/mod.rs`
  移除 `authz` 注册，改为 `access` 注册。
- `server/apps/api/src/composition/contexts/order.rs`
  在订单上下文装配 `AccessService` 及其同步 published-fact 适配器，替换现有 `authz` wiring。
- `server/apps/api/tests/architecture.rs`
  收紧 app-shell 守卫，禁止 `authz-*` 依赖残留并约束新 `Access` 组合方式。
- `server/apps/api/tests/context_skeleton_architecture.rs`
  扩充 workspace member 断言，覆盖 `access-infrastructure-sqlx`。
- `server/crates/identity-published/src/lib.rs`
  将当前占位 `SubjectRef` 升级为稳定 published subject contract 的导出入口。
- `server/crates/organization-published/src/lib.rs`
  保持 `Phase 2A` 已建立的 `BrandRef`、`StoreRef`、`StoreSummary` 作为唯一 canonical scope contract 导出入口，不允许本阶段重新收窄形状。
- `server/crates/access-published/src/lib.rs`
  把当前 `MembershipRef` 占位改成与 `Access` 语义一致的 published language 导出入口。
- `server/crates/access-domain/Cargo.toml`
  为真实 Access 领域模型补齐依赖。
- `server/crates/access-domain/src/lib.rs`
  导出真正的 Access 领域对象，而不是仅保留 skeleton 注释。
- `server/crates/access-application/Cargo.toml`
  增加 `async-trait`、`thiserror`、`identity-published`、`organization-published`、`access-published` 等依赖。
- `server/crates/access-application/src/lib.rs`
  导出新的 `AccessService`、ports、错误类型与 DTO。
- `server/crates/access-integration/Cargo.toml`
  如果需要保留轻量 translator / contract helper，则在此 crate 内补齐最小依赖；如果最终不承载运行时逻辑，也要把约束写清楚。
- `server/crates/access-integration/src/lib.rs`
  明确本阶段只承载 published-language translator/helper，不承载 event projector/outbox 逻辑。
- `server/crates/organization-application/src/lib.rs`
  导出新增的按 `store_id` 查询组织 scope fact 能力。
- `server/crates/organization-application/src/ports.rs`
  为 `Organization` 自身的 read/query seam 增加 `StoreReadRepository::get_by_id`。
- `server/crates/organization-application/src/module.rs`
  让 `StoreQueryService` 暴露 `get_by_id`。
- `server/crates/organization-infrastructure-sqlx/src/store_read_repository.rs`
  用 SQLx 实现 `get_by_id`，作为 `Organization` context 自身对外同步 published-fact provider 的底层读取。
- `server/crates/organization-infrastructure-sqlx/tests/repositories.rs`
  增补 `get_by_id` 的仓储测试。
- `server/crates/database-infrastructure-sqlx/src/bin/migration-info.rs`
  更新迁移清单断言，纳入新的 `access` additive migration。

### New files to create

- `server/apps/api/src/composition/contexts/access.rs`
  新的 `Access` 上下文注册入口，替代现有 `authz.rs`。
- `server/apps/api/src/composition/access_subject_provider.rs`
  在 app shell 中实现 `Access` 对 `Identity` published subject 事实的同步适配器。
- `server/apps/api/src/composition/access_store_scope_provider.rs`
  在 app shell 中实现 `Access` 对 `Organization` published store scope 事实的同步适配器；本阶段底层必须由 `Organization` 自身的 store query seam 提供。
- `server/apps/api/tests/access_boundary_architecture.rs`
  专门守卫 `authz-* -> access-*` 切换、`Access` 消费 published 事实边界，以及 app shell 同步例外只出现在组合根。
- `server/crates/identity-published/src/subject_ref.rs`
  `Identity` 对外稳定发布的主体引用与状态模型。
- `server/crates/identity-published/tests/contracts.rs`
  约束 `SubjectRef` 的稳定形状与最小行为。
- `server/crates/organization-published/src/store_ref.rs`
  `Organization` 对外稳定发布的门店作用域引用。
- `server/crates/organization-published/tests/contracts.rs`
  约束 `StoreRef` 的稳定形状。
- `server/crates/access-published/src/access_role_ref.rs`
  `Access` 对外稳定发布的角色语言。
- `server/crates/access-published/src/store_membership_ref.rs`
  `Access` 对外稳定发布的门店成员关系语言。
- `server/crates/access-published/tests/contracts.rs`
  守卫 `Access` published types 不再停留在 skeleton 命名。
- `server/crates/access-domain/src/access_role.rs`
  `Access` 内部角色枚举。
- `server/crates/access-domain/src/access_scope.rs`
  `Access` 内部作用域模型，至少覆盖平台级与门店级。
- `server/crates/access-domain/src/subject_access_grant.rs`
  `Access` 内部授权事实聚合根/实体的最小统一表示。
- `server/crates/access-application/src/error.rs`
  `Access` 应用错误。
- `server/crates/access-application/src/ports.rs`
  `Access` 应用层 ports，明确只消费 `identity-published` / `organization-published`。
- `server/crates/access-application/src/service.rs`
  `AccessService` 及当前阶段的核心判定方法，如 `can_manage_order`。
- `server/crates/access-application/tests/service.rs`
  `AccessService` 的行为测试。
- `server/crates/access-application/tests/architecture.rs`
  `Access` application 不直连其他上下文内部层的架构守卫。
- `server/crates/access-infrastructure-sqlx/Cargo.toml`
  新的持久化 crate，承接原 `authz-infrastructure-sqlx` 职责。
- `server/crates/access-infrastructure-sqlx/src/lib.rs`
  导出 SQLx 仓储实现。
- `server/crates/access-infrastructure-sqlx/src/db_roles.rs`
  新 `access` schema 下的 SQLx enum 映射。
- `server/crates/access-infrastructure-sqlx/src/repository.rs`
  `SqlxAccessGrantRepository`。
- `server/crates/access-infrastructure-sqlx/tests/repositories.rs`
  `Access` SQLx 仓储与 schema 约束测试。
- `server/crates/database-infrastructure-sqlx/migrations/202604050201_access.up.sql`
  additive `access` schema 迁移，创建新 schema 并从旧 `authz` 数据复制到新表。
- `server/crates/database-infrastructure-sqlx/migrations/202604050201_access.down.sql`
  additive `access` schema 回滚迁移。

### Existing files to delete after cutover

- `server/apps/api/src/composition/contexts/authz.rs`
- `server/crates/authz-domain/Cargo.toml`
- `server/crates/authz-domain/src/global_role.rs`
- `server/crates/authz-domain/src/store_membership.rs`
- `server/crates/authz-domain/src/store_role.rs`
- `server/crates/authz-domain/src/lib.rs`
- `server/crates/authz-application/Cargo.toml`
- `server/crates/authz-application/src/error.rs`
- `server/crates/authz-application/src/service.rs`
- `server/crates/authz-application/src/lib.rs`
- `server/crates/authz-infrastructure-sqlx/Cargo.toml`
- `server/crates/authz-infrastructure-sqlx/src/db_roles.rs`
- `server/crates/authz-infrastructure-sqlx/src/repository.rs`
- `server/crates/authz-infrastructure-sqlx/src/lib.rs`
- `server/crates/authz-infrastructure-sqlx/tests/repositories.rs`

### Existing files intentionally left unchanged in Phase 2B

- `server/crates/database-infrastructure-sqlx/migrations/202603150002_authz.up.sql`
- `server/crates/database-infrastructure-sqlx/migrations/202603150002_authz.down.sql`

这两条历史迁移已经属于已发布迁移轨迹。Phase 2B 不能通过改写历史来“伪装成一直叫 access”，必须新增 additive migration。

## Scope Check

本计划只覆盖 `server` workspace 中的 **Phase 2B: Access + Identity purification**，不混入以下内容：

1. 不做 event bus、outbox、projector network、projection rebuild。
2. 不做 `menu -> catalog` 命名迁移。
3. 不做 `order -> ordering + fulfillment` 实体与 crate 拆分。
4. 不新增 `Organization` 的新一轮领域重画或 schema ownership 迁移；这里只消费前置 `Phase 2A` 已落地的 `Organization` published facts，并按需补齐 `Organization` 自身对外 query seam。
5. 不做品牌级多角色体系扩展；当前仍保持“单品牌运行 + 平台管理员 + 门店角色”这一最小语义集合。

## Key Assumptions

1. Phase 2B 期间，`Identity` 的外部 published 主体 ID 与当前 `identity.users.id` 保持同值映射，即 `subject_id == user_id`；先纯化边界语言，不在本阶段重命名内部账户聚合。
2. `Phase 2A` 已经让 `Organization` 成为品牌/门店唯一真相源，并稳定发布 `BrandRef`、`StoreRef`、`StoreSummary`。如果 `Phase 2B` 需要按 `store_id` 同步查询 scope facts，只能在 `Organization` context 自身的 read/query seam 上补齐 `get_by_id`，不能把 `menu` 重新描述为事实来源。
3. 订单履约权限路径的业务行为应保持现状一致：`PlatformAdmin` 仍可管理任意门店订单，`StoreOwner`/`StoreStaff` 仍只在其门店作用域内生效。

## 方案对比与取舍

### 方案 A：直接把 `authz-*` crate 重命名为 `access-*`，保留旧 schema 与旧边界

不选。

原因：

1. 这只解决名字，不解决 `Identity` 仍缺少 published subject language、`Access` 仍无 published-fact 消费边界的问题。
2. 旧 `authz` 语义仍会继续以 `user_id`/`store_memberships` 的过渡语言向外扩散。
3. 后续接入 `Organization` 或事件主干时，还要再次返工。

### 方案 B：等待 `Organization`、事件总线、outbox 全部落地后再统一重画边界

不选。

原因：

1. 当前 `orders` 路径已经直接依赖 `authz`，继续等待只会让旧边界继续向新代码扩散。
2. 这会把“命名与职责纯化”问题和“事件主干”问题绑死在一起，实施风险和回滚面都过大。

### 方案 C：推荐方案，先做 Access/Identity 边界纯化，再保留同步白名单过渡

选这个。

原因：

1. `Access` 可以立即从 `authz` 语义脱身，形成稳定命名、稳定 schema 和稳定 application ports。
2. `Identity` 可以先发布最小 `SubjectRef`，把外部依赖从“用户资料视图”收回到“认证主体事实”。
3. `apps/api` 组合根可以作为显式同步例外白名单，暂时装配 `Identity`/`Organization` published-fact provider，而不把跨上下文耦合写进 `Access` crate 内。
4. additive migration 不会破坏已应用迁移，线上风险显著低于改写历史。

## Task 1: Freeze the published boundary for Subject and Store scope

**Files:**
- Modify: `server/crates/identity-published/src/lib.rs`
- Create: `server/crates/identity-published/src/subject_ref.rs`
- Create: `server/crates/identity-published/tests/contracts.rs`
- Modify: `server/crates/organization-published/tests/contracts.rs`
- Modify: `server/crates/access-published/src/lib.rs`
- Create: `server/crates/access-published/src/access_role_ref.rs`
- Create: `server/crates/access-published/src/store_membership_ref.rs`
- Create: `server/crates/access-published/tests/contracts.rs`

- [ ] **Step 1: 先写 published contract 测试，冻结 `Organization` 的 canonical scope family，同时让 `Identity/Access` 的占位契约先红**

```rust
use ordering_food_identity_published::{SubjectRef, SubjectStatus};

#[test]
fn subject_ref_tracks_subject_identity_and_status() {
    let subject = SubjectRef::new("subject-1", SubjectStatus::Active);

    assert_eq!(subject.subject_id(), "subject-1");
    assert_eq!(subject.status(), SubjectStatus::Active);
}
```

```rust
use ordering_food_organization_published::{BrandRef, StoreRef, StoreSummary};

#[test]
fn organization_scope_contract_keeps_phase_2a_canonical_shape() {
    let brand = BrandRef {
        brand_id: "brand-1".to_string(),
    };
    let store = StoreRef {
        store_id: "store-1".to_string(),
        brand_id: brand.brand_id.clone(),
    };
    let summary = StoreSummary {
        store_id: store.store_id.clone(),
        brand_id: store.brand_id.clone(),
        slug: "demo-kitchen".to_string(),
        name: "Demo Kitchen".to_string(),
        currency_code: "CNY".to_string(),
        timezone: "Asia/Shanghai".to_string(),
        status: "active".to_string(),
    };

    assert_eq!(summary.brand_id, "brand-1");
    assert_eq!(store.store_id, "store-1");
}
```

Run:

- `cd server && cargo test -p ordering-food-identity-published`
- `cd server && cargo test -p ordering-food-organization-published`
- `cd server && cargo test -p ordering-food-access-published`

Expected:

- `ordering-food-identity-published` 先 FAIL，因为当前 `SubjectRef` 只有 public field，占位 contract 没有稳定构造与状态语言。
- `ordering-food-organization-published` 应继续 PASS；如果这里失败，说明 `Phase 2A` 建立的 canonical scope contract 已经被后续计划意外漂移。
- `ordering-food-access-published` 先 FAIL，因为当前 `MembershipRef` 命名仍停留在 skeleton 阶段。

- [ ] **Step 2: 用稳定 published 语言替换占位类型**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubjectStatus {
    Active,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectRef {
    subject_id: String,
    status: SubjectStatus,
}
```

实现要求：

1. `Identity` published 只暴露 `subject_id` 和最小主体状态，不暴露 `UserReadModel`、资料字段、凭证字段。
2. `Organization` published contract family 在后续 phases 中统一保持为 `BrandRef + StoreRef + StoreSummary`；本阶段不能把 `StoreRef` 收窄成只包含 `store_id` 的新平行定义。
3. `Access` published 至少暴露稳定的角色语言与门店成员关系语言，不再使用含糊的 `MembershipRef` 占位命名。

- [ ] **Step 3: 重新运行 published crate 测试**

Run:

- `cd server && cargo test -p ordering-food-identity-published`
- `cd server && cargo test -p ordering-food-organization-published`
- `cd server && cargo test -p ordering-food-access-published`

Expected:

- 三个 crate 全部 PASS。

- [ ] **Step 4: 仅当用户明确授权时再提交**

```bash
git add \
  server/crates/identity-published \
  server/crates/organization-published \
  server/crates/access-published
git commit -m "refactor: define published contracts for access and identity seams"
```

## Task 2: Build the real Access domain and application around published facts

**Files:**
- Modify: `server/crates/access-domain/Cargo.toml`
- Modify: `server/crates/access-domain/src/lib.rs`
- Create: `server/crates/access-domain/src/access_role.rs`
- Create: `server/crates/access-domain/src/access_scope.rs`
- Create: `server/crates/access-domain/src/subject_access_grant.rs`
- Modify: `server/crates/access-application/Cargo.toml`
- Modify: `server/crates/access-application/src/lib.rs`
- Create: `server/crates/access-application/src/error.rs`
- Create: `server/crates/access-application/src/ports.rs`
- Create: `server/crates/access-application/src/service.rs`
- Create: `server/crates/access-application/tests/service.rs`
- Create: `server/crates/access-application/tests/architecture.rs`

- [ ] **Step 1: 先跑 GitNexus 影响分析，锁定旧 `authz` 行为的直接调用面**

Run:

- `gitnexus_impact({ repo: "ordering-food", target: "AuthorizationService", direction: "upstream" })`

Expected:

- 至少识别出 `server/apps/api/src/routes/orders.rs` 和 `server/apps/api/src/composition/contexts/order.rs` 是本阶段必须同步调整的主调用面。

- [ ] **Step 2: 先写失败的 Access 行为测试与架构测试**

```rust
#[tokio::test]
async fn disabled_subject_cannot_manage_order() {
    let service = build_service_with(
        vec![SubjectGrant::store_staff("subject-1", "store-1")],
        SubjectRef::new("subject-1", SubjectStatus::Disabled),
        Some(StoreRef {
            store_id: "store-1".to_string(),
            brand_id: "brand-1".to_string(),
        }),
    );

    assert!(!service.can_manage_order("subject-1", "store-1").await.unwrap());
}
```

```rust
#[test]
fn access_application_manifest_only_depends_on_published_language_of_other_contexts() {
    let manifest = std::fs::read_to_string("Cargo.toml").unwrap();

    assert!(!manifest.contains("ordering-food-identity-application"));
    assert!(!manifest.contains("ordering-food-organization-application"));
    assert!(manifest.contains("ordering-food-identity-published"));
    assert!(manifest.contains("ordering-food-organization-published"));
}
```

Run:

- `cd server && cargo test -p ordering-food-access-application --test service`
- `cd server && cargo test -p ordering-food-access-application --test architecture`

Expected:

- 两组测试先 FAIL，因为 `access-application` 目前还没有真实服务、ports 与架构约束。

- [ ] **Step 3: 实现最小但真实的 Access 领域与应用层**

```rust
#[async_trait]
pub trait SubjectFactsPort: Send + Sync {
    async fn get_subject(&self, subject_id: &str) -> Result<Option<SubjectRef>, ApplicationError>;
}

#[async_trait]
pub trait StoreScopeFactsPort: Send + Sync {
    async fn get_store(&self, store_id: &str) -> Result<Option<StoreRef>, ApplicationError>;
}

#[derive(Clone)]
pub struct AccessService {
    grants: Arc<dyn AccessGrantRepository>,
    subjects: Arc<dyn SubjectFactsPort>,
    stores: Arc<dyn StoreScopeFactsPort>,
}
```

实现要求：

1. `Access` 内部语言统一改为 `subject`、`scope`、`grant`、`role`，不再继续使用 `authz`/`user_global_roles` 的旧边界词汇。
2. `AccessService::can_manage_order` 必须先消费 `Identity` published `SubjectRef`，再消费 `Organization` published `StoreRef`，最后才读取本上下文授权事实。
3. 当前阶段角色集合保持最小兼容：`PlatformAdmin`、`StoreOwner`、`StoreStaff`。
4. `Access` application 不得依赖 `identity-application`、`organization-application`、`identity-infrastructure-*`、`organization-infrastructure-*`。

- [ ] **Step 4: 运行 Access 应用层测试**

Run:

- `cd server && cargo test -p ordering-food-access-domain`
- `cd server && cargo test -p ordering-food-access-application --test service`
- `cd server && cargo test -p ordering-food-access-application --test architecture`

Expected:

- 三组测试全部 PASS。

- [ ] **Step 5: 仅当用户明确授权时再提交**

```bash
git add \
  server/crates/access-domain \
  server/crates/access-application
git commit -m "refactor: implement access domain and application services"
```

## Task 3: Replace `authz` persistence with `access-infrastructure-sqlx` and an additive schema migration

**Files:**
- Modify: `server/Cargo.toml`
- Create: `server/crates/access-infrastructure-sqlx/Cargo.toml`
- Create: `server/crates/access-infrastructure-sqlx/src/lib.rs`
- Create: `server/crates/access-infrastructure-sqlx/src/db_roles.rs`
- Create: `server/crates/access-infrastructure-sqlx/src/repository.rs`
- Create: `server/crates/access-infrastructure-sqlx/tests/repositories.rs`
- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050201_access.up.sql`
- Create: `server/crates/database-infrastructure-sqlx/migrations/202604050201_access.down.sql`
- Modify: `server/crates/database-infrastructure-sqlx/src/bin/migration-info.rs`

- [ ] **Step 1: 先写失败的 SQLx 仓储测试与迁移清单测试**

```rust
#[sqlx::test(migrator = "MIGRATOR")]
async fn repository_reads_store_roles_from_access_schema(pool: PgPool) {
    let repository = SqlxAccessGrantRepository::new(pool.clone());

    let roles = repository
        .get_store_roles("00000000-0000-0000-0000-000000000001", "00000000-0000-0000-0000-000000000101")
        .await
        .unwrap();

    assert!(roles.is_empty());
}
```

Run:

- `cd server && cargo test -p ordering-food-access-infrastructure-sqlx`
- `cd server && cargo test -p ordering-food-database-infrastructure-sqlx --bin migration-info`

Expected:

- `ordering-food-access-infrastructure-sqlx` 先 FAIL，因为 package 还不存在。
- `migration-info` 先 FAIL，因为当前测试只知道 `baseline`、`ordering`、`authz` 三条迁移。

- [ ] **Step 2: 创建新的 persistence crate 和 additive migration**

```sql
CREATE SCHEMA IF NOT EXISTS access;

CREATE TYPE access.global_role AS ENUM ('platform_admin');
CREATE TYPE access.store_role AS ENUM ('store_owner', 'store_staff');

CREATE TABLE access.subject_global_roles (
    subject_id UUID NOT NULL,
    role access.global_role NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (subject_id, role)
);
```

实现要求：

1. 新 schema 必须命名为 `access`，列名改成 `subject_id`，不能继续把外部边界语言写成 `user_id`。
2. `202603150002_authz.*` 不改动；`202604050201_access.up.sql` 内部负责把旧 `authz` 数据复制到新 `access` 表。
3. `down.sql` 只回滚新增的 `access` schema，不回滚历史 `authz` schema。
4. 新 SQLx crate 名称统一使用当前仓库风格 `access-infrastructure-sqlx`，不要只在 `Access` 上引入特例化后缀。

- [ ] **Step 3: 运行新的 persistence 与 migration 测试**

Run:

- `cd server && cargo test -p ordering-food-access-infrastructure-sqlx`
- `cd server && cargo test -p ordering-food-database-infrastructure-sqlx --bin migration-info`

Expected:

- 两组测试全部 PASS。

- [ ] **Step 4: 仅当用户明确授权时再提交**

```bash
git add \
  server/Cargo.toml \
  server/crates/access-infrastructure-sqlx \
  server/crates/database-infrastructure-sqlx/migrations/202604050201_access.up.sql \
  server/crates/database-infrastructure-sqlx/migrations/202604050201_access.down.sql \
  server/crates/database-infrastructure-sqlx/src/bin/migration-info.rs
git commit -m "refactor: move access persistence to new schema and crate"
```

## Task 4: Wire app-shell sync providers and switch order authorization from `authz` to `Access`

**Files:**
- Modify: `server/apps/api/Cargo.toml`
- Create: `server/apps/api/src/composition/contexts/access.rs`
- Create: `server/apps/api/src/composition/access_subject_provider.rs`
- Create: `server/apps/api/src/composition/access_store_scope_provider.rs`
- Modify: `server/apps/api/src/composition/contexts/mod.rs`
- Modify: `server/apps/api/src/composition/contexts/order.rs`
- Modify: `server/apps/api/src/http.rs`
- Modify: `server/apps/api/src/routes/auth.rs`
- Modify: `server/apps/api/src/routes/orders.rs`
- Modify: `server/crates/organization-application/src/lib.rs`
- Modify: `server/crates/organization-application/src/ports.rs`
- Modify: `server/crates/organization-application/src/module.rs`
- Modify: `server/crates/organization-infrastructure-sqlx/src/store_read_repository.rs`
- Modify: `server/crates/organization-infrastructure-sqlx/tests/repositories.rs`

- [ ] **Step 1: 先跑 GitNexus 影响分析，确认 app-shell 改线的直接影响面**

Run:

- `gitnexus_impact({ repo: "ordering-food", target: "register_authz", direction: "upstream" })`
- `gitnexus_impact({ repo: "ordering-food", target: "register_order", direction: "upstream" })`
- `gitnexus_impact({ repo: "ordering-food", target: "AuthenticatedUser", direction: "upstream" })`

Expected:

- 至少锁定 `server/apps/api/src/composition/contexts/mod.rs`、`server/apps/api/src/composition/contexts/order.rs`、`server/apps/api/src/routes/orders.rs`、`server/apps/api/src/routes/auth.rs` 为 d=1 变更面。

- [ ] **Step 2: 先写失败的门店 scope 读取测试与订单路由授权测试**

```rust
#[tokio::test]
async fn get_by_id_returns_store_when_store_exists() {
    let store = repository.get_by_id(&StoreId::new(store_id.to_string())).await.unwrap();
    assert!(store.is_some());
}
```

```rust
#[tokio::test]
async fn store_staff_subject_can_accept_order_through_access_service() {
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
```

Run:

- `cd server && cargo test -p ordering-food-organization-infrastructure-sqlx --test repositories`
- `cd server && cargo test -p ordering-food-api --lib store_staff_subject_can_accept_order_through_access_service`

Expected:

- `Organization` 仓储测试先 FAIL，因为 `StoreReadRepository::get_by_id` 尚不存在。
- API 路由测试先 FAIL，因为订单路由仍在注入 `AuthorizationService`。

- [ ] **Step 3: 在组合根内装配同步 published-fact provider，并切换订单权限路径**

```rust
pub struct AuthenticatedSubject {
    pub subject_id: String,
}
```

```rust
let access = Arc::new(AccessService::new(
    access_repository,
    subject_facts_port,
    store_scope_facts_port,
));
```

实现要求：

1. `apps/api` 组合根是本阶段唯一允许保留同步跨上下文白名单的地方。
2. `Access` 对 `Identity` 的消费必须通过 `identity-published::SubjectRef` 语义完成，不能把 `UserReadModel` 直接传进 `AccessService`。
3. `Access` 对 `Organization` 的消费必须通过 `organization-published::StoreRef` 语义完成；本阶段底层实现只能调用 `Organization` 自身的 query seam，不允许再把 `menu` 兼容视图或 `menu` 读仓储上升为事实来源。
4. `server/apps/api/src/http.rs` 只改变外部认证主体语义，不在本阶段重命名 Identity 内部 `UserId`。

- [ ] **Step 4: 运行 app shell、Organization query seam 与订单路由测试**

Run:

- `cd server && cargo test -p ordering-food-organization-application`
- `cd server && cargo test -p ordering-food-organization-infrastructure-sqlx --test repositories`
- `cd server && cargo test -p ordering-food-api --lib store_staff_subject_can_accept_order_through_access_service`
- `cd server && cargo test -p ordering-food-api --lib platform_admin_subject_can_accept_any_store_order`

Expected:

- 四组测试全部 PASS。

- [ ] **Step 5: 仅当用户明确授权时再提交**

```bash
git add \
  server/apps/api/Cargo.toml \
  server/apps/api/src/http.rs \
  server/apps/api/src/routes/auth.rs \
  server/apps/api/src/routes/orders.rs \
  server/apps/api/src/composition \
  server/crates/organization-application/src/lib.rs \
  server/crates/organization-application/src/ports.rs \
  server/crates/organization-application/src/module.rs \
  server/crates/organization-infrastructure-sqlx/src/store_read_repository.rs \
  server/crates/organization-infrastructure-sqlx/tests/repositories.rs
git commit -m "refactor: wire access service through app shell providers"
```

## Task 5: Remove the old `authz` surface and add architecture guards

**Files:**
- Modify: `server/Cargo.toml`
- Modify: `server/apps/api/Cargo.toml`
- Modify: `server/apps/api/tests/architecture.rs`
- Modify: `server/apps/api/tests/context_skeleton_architecture.rs`
- Create: `server/apps/api/tests/access_boundary_architecture.rs`
- Delete: `server/apps/api/src/composition/contexts/authz.rs`
- Delete: `server/crates/authz-domain/Cargo.toml`
- Delete: `server/crates/authz-domain/src/global_role.rs`
- Delete: `server/crates/authz-domain/src/store_membership.rs`
- Delete: `server/crates/authz-domain/src/store_role.rs`
- Delete: `server/crates/authz-domain/src/lib.rs`
- Delete: `server/crates/authz-application/Cargo.toml`
- Delete: `server/crates/authz-application/src/error.rs`
- Delete: `server/crates/authz-application/src/service.rs`
- Delete: `server/crates/authz-application/src/lib.rs`
- Delete: `server/crates/authz-infrastructure-sqlx/Cargo.toml`
- Delete: `server/crates/authz-infrastructure-sqlx/src/db_roles.rs`
- Delete: `server/crates/authz-infrastructure-sqlx/src/repository.rs`
- Delete: `server/crates/authz-infrastructure-sqlx/src/lib.rs`
- Delete: `server/crates/authz-infrastructure-sqlx/tests/repositories.rs`

- [ ] **Step 1: 先写失败的架构守卫，防止 `authz` 继续残留**

```rust
#[test]
fn app_shell_no_longer_references_authz_crates() {
    let manifest = std::fs::read_to_string("Cargo.toml").unwrap();

    assert!(!manifest.contains("ordering-food-authz-application"));
    assert!(!manifest.contains("ordering-food-authz-domain"));
    assert!(!manifest.contains("ordering-food-authz-infrastructure-sqlx"));
}
```

```rust
#[test]
fn access_application_sources_do_not_import_foreign_internal_layers() {
    let source = std::fs::read_to_string("../src/ports.rs").unwrap();

    assert!(!source.contains("ordering_food_identity_application"));
    assert!(!source.contains("ordering_food_organization_application"));
}
```

Run:

- `cd server && cargo test -p ordering-food-api --test access_boundary_architecture`
- `cd server && cargo test -p ordering-food-access-application --test architecture`

Expected:

- 两组测试先 FAIL，因为旧 `authz` crate 和依赖还没有彻底移除。

- [ ] **Step 2: 删除旧 `authz` 文件并收紧 workspace / app-shell 依赖**

实现要求：

1. 删除旧 `authz-*` crate 文件时，同步从 `server/Cargo.toml` 与 `server/apps/api/Cargo.toml` 中移除依赖与 members。
2. 保留历史 `authz` SQL migration 文件，不把“删 crate”误伤成“删历史迁移”。
3. `apps/api/tests/access_boundary_architecture.rs` 要明确守卫“同步白名单只允许出现在组合根，不允许写进 Access crate”。

- [ ] **Step 3: 重新运行架构守卫**

Run:

- `cd server && cargo test -p ordering-food-api --test architecture`
- `cd server && cargo test -p ordering-food-api --test context_skeleton_architecture`
- `cd server && cargo test -p ordering-food-api --test access_boundary_architecture`
- `cd server && cargo test -p ordering-food-access-application --test architecture`

Expected:

- 四组测试全部 PASS。

- [ ] **Step 4: 仅当用户明确授权时再提交**

```bash
git add server/Cargo.toml server/apps/api/Cargo.toml server/apps/api/tests
git rm \
  server/apps/api/src/composition/contexts/authz.rs \
  server/crates/authz-domain \
  server/crates/authz-application \
  server/crates/authz-infrastructure-sqlx
git commit -m "refactor: remove legacy authz surface after access cutover"
```

## Task 6: Full verification, diff audit, and handoff

**Files:**
- No additional file edits in this task.

- [ ] **Step 1: 运行完整测试矩阵**

Run:

- `cd server && cargo test -p ordering-food-identity-published`
- `cd server && cargo test -p ordering-food-organization-published`
- `cd server && cargo test -p ordering-food-access-published`
- `cd server && cargo test -p ordering-food-access-domain`
- `cd server && cargo test -p ordering-food-access-application`
- `cd server && cargo test -p ordering-food-access-infrastructure-sqlx`
- `cd server && cargo test -p ordering-food-organization-application`
- `cd server && cargo test -p ordering-food-organization-infrastructure-sqlx`
- `cd server && cargo test -p ordering-food-database-infrastructure-sqlx --bin migration-info`
- `cd server && cargo test -p ordering-food-api --tests`

Expected:

- 全部 PASS。

- [ ] **Step 2: 运行针对本阶段的 clippy 校验**

Run:

- `cd server && cargo clippy -p ordering-food-access-domain -p ordering-food-access-application -p ordering-food-access-infrastructure-sqlx -p ordering-food-api --tests -- -D warnings`

Expected:

- PASS，且不再出现对 `authz-*` 的警告引用。

- [ ] **Step 3: 运行变更范围审计，确认只命中预期边界**

Run:

- `cd server && git diff --stat`
- `gitnexus_detect_changes({ repo: "ordering-food", scope: "all" })`

Expected:

- 改动只覆盖 `Access`、`Identity published`、`Organization` canonical published seam、`orders` 组合根 wiring、`Organization` 自身的最小 query seam，以及新的 additive `access` migration。

- [ ] **Step 4: 仅当用户明确授权时再提交**

```bash
git add server
git commit -m "refactor: complete backend ddd phase 2b access and identity purification"
```

## Done Criteria

Phase 2B 完成时，必须同时满足以下结果：

1. 工作区内不再存在 `authz-domain`、`authz-application`、`authz-infrastructure-sqlx` 运行时代码依赖。
2. `Access` 成为唯一承载“谁在什么作用域内拥有什么能力”的上下文。
3. `Identity` 对外只通过 `SubjectRef` 这类认证主体语言暴露给其他上下文消费。
4. `Access` 只消费 `identity-published` 与 `organization-published`，不直接依赖外部上下文内部模型。
5. `apps/api` 是本阶段唯一允许保留同步白名单适配器的位置。
6. `access` schema 通过 additive migration 建立完成，历史 `authz` 迁移文件保持不变。
7. 不引入 event bus、outbox、projector network，也不提前把本阶段扩张成 `Catalog` 或 `Fulfillment` 迁移。

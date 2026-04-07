# Backend DDD Backend Plan Index

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement these plans task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Provide one execution index for the full backend DDD migration plan so every later implementation step follows the approved dependency order instead of treating the plan set as independent documents.

**Architecture:** The backend migration is intentionally split into multiple executable plans because the target architecture spans several bounded contexts and two different classes of work: boundary redraw and cross-context collaboration infrastructure. The execution order is strict: first establish context ownership, then redraw business boundaries, then build the event spine, and only then harden governance and long-term guardrails.

**Tech Stack:** Rust 2024, Cargo workspace, Tokio, Axum, SQLx, published contracts, ACL, integration read models, Postgres outbox, tokio dispatcher / projector, architecture tests, SQL migrations.

---

## Plan Set

### 已完成或已落地

- [ ] `Phase 1` 边界骨架
  Plan: [2026-04-05-backend-ddd-phase-1-boundary-skeleton.md](/Users/ggg/private/ordering-food/docs/superpowers/plans/2026-04-05-backend-ddd-phase-1-boundary-skeleton.md)
  Status note: 代码已在当前分支落地，但未 commit。

### 后续执行顺序

1. [ ] `Phase 2A` Organization foundation  
   Plan: [2026-04-05-backend-ddd-phase-2a-organization-foundation.md](/Users/ggg/private/ordering-food/docs/superpowers/plans/2026-04-05-backend-ddd-phase-2a-organization-foundation.md)  
   Why first: `Store / Brand` 真相必须先从 `menu` 抽出来，否则后续上下文都只能继续依赖假边界。

2. [ ] `Phase 2B` Access + Identity purification  
   Plan: [2026-04-05-backend-ddd-phase-2b-access-and-identity-purification.md](/Users/ggg/private/ordering-food/docs/superpowers/plans/2026-04-05-backend-ddd-phase-2b-access-and-identity-purification.md)  
   Depends on: `Phase 2A`  
   Why second: `Access` 需要消费稳定的组织 scope facts；同时必须先把认证与授权边界分开，后续 `Ordering/Fulfillment` 才不会继续绑在 `authz` 上。

3. [ ] `Phase 2C` Catalog migration  
   Plan: [2026-04-05-backend-ddd-phase-2c-catalog-migration.md](/Users/ggg/private/ordering-food/docs/superpowers/plans/2026-04-05-backend-ddd-phase-2c-catalog-migration.md)  
   Depends on: `Phase 2A`  
   Why third: `Catalog` 需要基于 `Organization` 的 store/brand scope，而不是继续自带 store 真相。

4. [ ] `Phase 2D` Ordering / Fulfillment split  
   Plan: [2026-04-05-backend-ddd-phase-2d-ordering-fulfillment-split.md](/Users/ggg/private/ordering-food/docs/superpowers/plans/2026-04-05-backend-ddd-phase-2d-ordering-fulfillment-split.md)  
   Depends on: `Phase 2A` + `Phase 2B` + `Phase 2C`  
   Why fourth: 订单商业语义与履约语义拆分，必须建立在稳定的组织、授权、目录边界之上。

5. [ ] `Phase 3` Event spine and projections  
   Plan: [2026-04-05-backend-ddd-phase-3-event-spine-and-projections.md](/Users/ggg/private/ordering-food/docs/superpowers/plans/2026-04-05-backend-ddd-phase-3-event-spine-and-projections.md)  
   Depends on: `Phase 2A` + `Phase 2B` + `Phase 2C` + `Phase 2D`  
   Why fifth: 只有边界和 owned schema 稳定后，published language / ACL / integration read model / outbox 才有正确落点。

6. [ ] `Phase 4` Purification and governance  
   Plan: [2026-04-05-backend-ddd-phase-4-purification-and-governance.md](/Users/ggg/private/ordering-food/docs/superpowers/plans/2026-04-05-backend-ddd-phase-4-purification-and-governance.md)  
   Depends on: `Phase 3`  
   Why last: 这一阶段收紧同步例外、schema ownership、投影重建、幂等与契约演进，属于长期稳态治理，不应在边界未定型时提前做。

## Execution Rules

- [ ] 默认执行模式使用 `Subagent-Driven`。
- [ ] 架构敏感任务、计划复审、跨 context 依赖判断优先使用 `gpt-5.4 xhigh`。
- [ ] 任何代码实现前都要先引用对应计划文档，不允许跳步骤。
- [ ] 任何 commit / push 都必须在当次再次得到用户明确授权。
- [ ] `Phase 3` 之前允许极少数显式同步白名单例外，但必须在对应计划中落明文债务说明。
- [ ] 所有 SQL migration 文件名必须沿整套计划保持全局唯一且顺序单调；本计划集保留的版本前缀序列为 `202604050101`、`202604050201`、`202604050301/302`、`202604050401/402`、`202604050501+`，实际实施时不得复用已占用版本号。

## Stop Conditions

- [ ] 如果 `Phase 2A` 未完成，不启动 `2B/2C/2D` 的实际代码实施。
- [ ] 如果 `2B` 与 `2C` 的 published contract 未稳定，不启动 `2D` 的最终 cutover。
- [ ] 如果 `Phase 3` 的 outbox / projector 没有真实落地，不进入 `Phase 4` 的同步例外收紧。
- [ ] 如果任一阶段的 architecture guards 还没绿，不开始下一阶段。

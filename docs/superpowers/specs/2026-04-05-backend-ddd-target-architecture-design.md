# 后端目标架构设计

- 日期：2026-04-05
- 主题：ordering-food 后端 DDD 纯化目标架构
- 范围：仅 `server` workspace
- 性质：目标态架构总规，不是一步到位的实施方案

## 1. 背景与目标

当前后端已经具备模块化单体的雏形，但边界仍然带有明显的过渡性特征：

- 平台公共抽象曾由业务上下文定义，依赖方向不自然
- `menu`、`authz`、`order` 等命名不能准确表达长期稳定的领域能力
- 认证、授权、组织、目录、订单、履约之间的业务边界尚未完全纯化
- 路由层与基础设施层的隔离已经存在，但跨上下文语言、ACL、投影、本地读模型尚未成为一等公民

本设计的目标不是对现有结构做局部修补，而是为后端定义一份清晰、彻底、可长期演进的 DDD 目标架构，使后续所有实施计划都围绕同一个终态收敛。

本设计采用的核心取向如下：

- 后端按能力型 `Bounded Context` 切分，而不是按角色或页面切分
- 当前系统按单品牌运行，但领域模型不能堵死未来升级到多品牌的道路
- 每个上下文拥有自己的数据、语言、事件和投影
- 上下文内部事务强一致
- 上下文之间默认最终一致
- 跨上下文协作默认通过 `published language + ACL + 事件 + 本地投影`
- 同步协作不是默认路径，只能作为极少数显式白名单例外

## 2. 业务本体与设计原则

### 2.1 业务本体

目标业务本体定义为：

> 当前按单品牌运行、未来可升级到多品牌的多门店品牌系统。

这意味着：

- `Brand` 是重要领域概念，即使当前运行时只存在单品牌，也不能在模型层彻底删除
- `Store` 是组织与经营的核心载体
- 顾客、门店人员、品牌人员只是消费视图，不是顶层边界切分依据

### 2.2 DDD 目标原则

本设计遵守以下 DDD 原则：

1. `Bounded Context` 由业务能力决定，而不是由 UI 页面、接口路径或角色入口决定。
2. 统一语言必须在上下文边界内成立，跨上下文协作必须通过显式发布语言完成。
3. 每个上下文应能独立解释自己的模型、事务与数据所有权。
4. 共享层必须极小且技术中立，不能演化为“全局通用业务模型层”。
5. 架构边界必须可自动化验证，不能只依赖人工记忆和代码评审。

## 3. 目标 Bounded Context 划分

目标态将后端划分为六个能力型上下文。

### 3.1 Identity

负责：

- 账户主体
- 凭证
- 登录方式
- 会话与 token 生命周期

不负责：

- 品牌或门店成员关系
- 业务角色
- 门店作用域权限

一句话定义：`Identity` 只回答“你是谁，以及如何被认证”。

### 3.2 Access

负责：

- 角色分配
- 品牌与门店范围内的成员关系
- 授权策略判定

依赖的外部事实：

- `Identity` 发布的主体引用
- `Organization` 发布的品牌与门店作用域引用

一句话定义：`Access` 只回答“你在什么作用域内有什么能力”。

### 3.3 Organization

负责：

- 品牌
- 门店
- 门店生命周期
- 品牌与门店的组织边界

一句话定义：`Organization` 提供经营载体与组织作用域真相。

### 3.4 Catalog

负责：

- 品牌目录
- 门店可售目录
- 分类、菜品、定价
- 可售状态与展示规则

不使用 `menu` 命名，原因是目标能力不只是“菜单展示”，而是完整目录与售卖规则。

### 3.5 Ordering

负责：

- 下单命令
- 订单成立
- 顾客侧商业订单语义
- 订单快照与商业状态

一句话定义：`Ordering` 负责“订单这件商业事实何时成立、以什么快照成立”。

### 3.6 Fulfillment

负责：

- 接单
- 备餐
- 可取餐
- 完成 / 拒单等履约推进

一句话定义：`Fulfillment` 负责“订单成立后，门店如何执行它”。

## 4. 目标物理结构

### 4.1 顶层 workspace 结构

目标态的 `server` workspace 分为三类内容：

1. 平台与公共抽象
2. 按上下文成组组织的业务 crate
3. 应用壳层

### 4.2 平台与公共抽象

建议引入下列平台级 crate：

- `platform-kernel`
- `platform-events`
- `platform-messaging`
- `platform-persistence`

其中最关键的是 `platform-kernel`，用于承载真正技术中立、跨上下文稳定共识的抽象，例如：

- `Clock`
- `IdGenerator`
- `EventId`
- `MessageId`
- `CorrelationId`
- `CausationId`

这些抽象不得由任何业务上下文定义。

### 4.3 每个上下文的标准 crate 组

每个上下文统一拆成以下职责层：

- `<context>-domain`
- `<context>-application`
- `<context>-published`
- `<context>-integration`
- `<context>-infrastructure-sqlx`
- `<context>-infrastructure-runtime`（仅当某个上下文确实拥有独立 runtime/consumer/adapter 进程职责时再引入，不作为默认必选项）

含义如下：

#### `*-domain`

只放：

- 聚合
- 值对象
- 领域服务
- 领域事件定义
- 上下文内部统一语言

禁止依赖：

- Web 框架
- 数据库
- Redis
- 其他上下文

#### `*-application`

只放：

- 用例
- 事务边界
- ports
- 应用编排

允许依赖：

- 本上下文 `domain`
- 平台级抽象
- 其他上下文的 `published`

禁止依赖：

- 其他上下文的 `domain`
- 其他上下文的 `application`
- 任意基础设施实现

#### `*-published`

只放本上下文对外公开的稳定语言：

- published events
- published query models
- published command contracts
- 稳定外部引用类型

#### `*-integration`

只放：

- ACL translator
- event projector
- integration fact adapter
- 本地投影更新器

它是唯一允许处理“外部 published language 进入本上下文”的内部层。

#### `*-infrastructure-sqlx`

只放：

- repository 实现
- transaction manager 实现
- projection store 实现
- schema 读写映射

#### `*-infrastructure-runtime`

只放运行时 provider，例如：

- token provider
- password hasher
- event bus adapter
- message transport adapter

### 4.4 应用壳层

`apps/api` 继续作为唯一 HTTP 入口，但只承担：

- route adapter
- request/response mapping
- composition root
- OpenAPI / export-ts
- runtime wiring

`apps/api` 不再承载事实上的业务实现。

## 5. 依赖方向总规

目标态依赖方向必须满足下列约束：

1. `apps/api` 只能依赖各上下文的 `application`、`published`，以及装配所需基础设施实现。
2. `domain` 不能依赖任何其他上下文。
3. `application` 不能依赖其他上下文的 `domain`、`application`、`infrastructure-*`。
4. 跨上下文依赖只能指向对方的 `published`。
5. 外部 published language 进入本上下文后，必须先经过 `integration`。
6. 平台公共抽象不能寄生在业务上下文中。

## 6. 数据所有权与 Schema 规则

### 6.1 总原则

每个 `Bounded Context` 只拥有并写入自己的数据。

其他上下文：

- 不允许直接查询其内部表
- 不允许写入其内部表
- 不允许通过 SQL join 跨越上下文边界实现业务逻辑

### 6.2 目标 schema 划分

数据库 schema 与上下文一一对应：

- `identity`
- `access`
- `organization`
- `catalog`
- `ordering`
- `fulfillment`

### 6.3 本地投影的归属规则

integration read model 也是本上下文拥有的数据，因此必须落在投影拥有方自己的 schema 下。

例如：

- `ordering` 为下单判定维护的商品投影，应位于 `ordering.*`
- `fulfillment` 为履约工作台维护的订单投影，应位于 `fulfillment.*`

不得将“供别人使用的只读表”建在来源上下文 schema 中作为妥协方案。

## 7. 跨上下文协作模型

### 7.1 默认协作方式

跨上下文协作默认使用：

- published language
- ACL
- published event
- integration read model

默认不使用：

- 跨 schema 直查
- 共享数据库事务
- 直接复用外部内部模型

### 7.2 一致性原则

- 上下文内部事务强一致
- 上下文之间默认最终一致
- 默认通过事件和投影收敛，不追求单事务内立即一致

### 7.3 同步例外

同步 published interface 不是禁用，但只能作为显式白名单例外，且必须满足：

1. 当前命令必须立即决策
2. 无法接受事件传播窗口
3. 问题本身不意味着边界切错
4. 依赖的是外部 `published` 契约，不是内部模型

所有同步例外必须被单独列出和审计，不能默许扩散。

### 7.4 Phase 2 临时同步例外白名单

Phase 2 为了先完成 `Ordering` / `Fulfillment` 的切边界，允许存在且仅允许存在以下两条临时同步例外：

- `TemporarySyncFulfillmentWorkflowGateway`
- `TemporarySyncFulfillmentBootstrapGateway`

这两条路径的定位必须明确：

- 它们是 `Phase 2` 的临时同步例外，用于边界切分过程中的过渡协作
- 它们不是长期协作模型，不代表目标态下上下文之间应以同步方式耦合
- 它们都带有明确的技术债属性，必须在后续实施和评审中单独可见
- `Phase 3` 必须用事件协作替换 这两条同步路径，届时不应再保留它们作为常规协作入口

额外约束如下：

- `TemporarySyncFulfillmentWorkflowGateway` 只允许承担履约工作流推进前的商业状态即时校验
- `TemporarySyncFulfillmentBootstrapGateway` 只允许承担新订单建立后与顾客取消后的履约侧 bootstrap
- 不允许第三条 sync path 在未重开架构方案评审前进入代码库

## 8. Published Language、ACL 与本地投影

### 8.1 Published Language

`published` 层表示：

> 本上下文愿意对外承诺稳定性的语言。

它不是内部模型原样暴露，而是边界化的外部表达。

例如目标态可出现的 published 语言：

- `StoreSummary`
- `StoreOrderingAvailability`
- `SellableItem`
- `CatalogPriceFact`
- `OrderPlaced`
- `OrderCommercialStateChanged`
- `FulfillmentStatusChanged`
- `RoleAssignmentChanged`

### 8.2 ACL

`ACL` 是消费方的翻译边界，作用是：

- 将外部上下文的 published language 翻译为本地可用语义
- 防止外部模型污染本地 application / domain

ACL 必须归属于 `integration` 层。

### 8.3 Integration Read Model

integration read model 定义为：

> 本上下文为了自己的查询、判定或编排目的，依据外部 published facts 在本地维护的只读视图模型。

它具有以下特征：

- 数据归属属于消费方上下文
- 来源于外部 published event 或 published query language
- 经 ACL 翻译后进入本地
- 只服务本上下文自己的业务，不代表外部上下文的真相

目标态中，integration read model 作为默认推荐协作方式存在。

## 9. 主干上下文协作关系

### 9.1 Identity

发布：

- 主体存在事实
- 主体状态变化
- 会话与认证相关事实

### 9.2 Access

消费：

- `Identity` 主体引用
- `Organization` 作用域引用

发布：

- 成员关系事实
- 角色分配事实
- 授权变更事实

### 9.3 Organization

发布：

- 品牌存在事实
- 门店存在事实
- 门店状态变化

### 9.4 Catalog

消费：

- `Organization` 门店与品牌事实

发布：

- 可售目录事实
- 价格变化事实
- 可售状态变化事实

### 9.5 Ordering

消费：

- `Organization` 的门店可营业事实
- `Catalog` 的商品与价格事实
- `Access` 的授权事实（如某些后台命令需要）

发布：

- 订单已创建
- 订单商业状态变化
- 订单已取消

### 9.6 Fulfillment

消费：

- `Ordering` 的订单事实
- `Organization` 的门店事实
- `Access` 的门店授权事实

发布：

- 已接单
- 开始备餐
- 可取餐
- 已完成
- 已拒单

## 10. 命名总规

### 10.1 Context 命名

固定使用：

- `Identity`
- `Access`
- `Organization`
- `Catalog`
- `Ordering`
- `Fulfillment`

必须淘汰的旧倾向包括：

- `authz`
- `menu`
- `order` 作为未拆分整体

### 10.2 Crate 命名

统一采用：

- `<context>-domain`
- `<context>-application`
- `<context>-published`
- `<context>-integration`
- `<context>-infrastructure-sqlx`
- `<context>-infrastructure-runtime`（可选，不默认创建）

### 10.3 Projection / Integration 命名

integration 层对象应显式表达其职责，例如：

- `CatalogSellableItemTranslator`
- `OrganizationStoreProjectionUpdater`
- `AccessMembershipFactMapper`
- `OrderingEventProjector`
- `FulfillmentOrderFactAdapter`

禁止使用含糊命名如：

- `util`
- `helper`
- `common_service`

## 11. 测试与架构守卫体系

目标态必须建立覆盖以下层面的自动化守卫：

### 11.1 crate 依赖守卫

验证：

- `apps/api` 不越过边界
- `application` 不直连其他上下文内部层
- `domain` 不污染技术依赖
- `integration` 是唯一外部语言吸收层

### 11.2 schema ownership 守卫

验证：

- persistence crate 只能操作本上下文 schema
- 其他上下文不能通过 SQL 访问他方 schema
- 本地投影位于拥有方自己的 schema

### 11.3 published / integration 守卫

验证：

- published contract 只从 `*-published` 导出
- 外部 published language 只能经 `integration` 进入本上下文
- projection 更新逻辑归属 `integration`

### 11.4 事件与投影守卫

验证：

- 关键跨上下文事实必须有明确 published event
- 投影处理具有幂等性
- 投影可重建
- 事件消费具备版本演进策略

### 11.5 同步例外白名单守卫

验证：

- 所有跨上下文同步调用必须列入白名单
- 白名单项说明存在理由
- 未授权同步协作视为架构违规

## 12. 技术选型倾向

### 12.1 总体原则

事件驱动不应将基础设施能力完全手写，但也不能把架构核心交给某个“全能框架”。

因此本设计采用以下技术原则：

1. 基础设施层尽量使用成熟、主流、长期维护的库。
2. `published language`、`ACL`、`outbox`、`projector`、`integration read model` 属于架构核心，必须由本项目显式定义。
3. 不引入统治性的 DDD / CQRS / Event Sourcing 框架作为核心骨架。

### 12.2 保留的基础栈

目标态默认继续沿用以下主流基础库：

- `tokio`
- `axum`
- `tower`
- `tower-http`
- `serde`
- `serde_json`
- `sqlx`
- `tracing`
- `tracing-subscriber`
- `config`
- `redis`
- `utoipa`
- `uuid`
- `time`
- `thiserror`
- `anyhow`
- `jsonwebtoken`
- `argon2`

这些库负责：

- Web 与 runtime
- 数据访问
- 序列化
- 配置
- 认证与密码学
- 基础错误与时间/ID 能力

目标架构的纯度来源于边界设计，不来源于替换主框架。

### 12.3 事件驱动的默认落地方式

目标态默认采用：

- `Postgres outbox`
- `sqlx`
- `tokio` internal dispatcher / projector

作为事件驱动的默认实现方式。

含义是：

1. 业务事务与 outbox 记录在同一数据库事务中提交。
2. 事件分发与投影更新由项目内部明确实现的 dispatcher / projector 负责。
3. 事件语义、幂等规则、重试策略、ACL 翻译与投影规则由本项目定义。

这样做的原因是：

- 当前目标是纯化模块化单体的边界，而不是立刻引入外部消息系统复杂度。
- `outbox + projector` 足以支撑默认的最终一致架构。
- 架构核心不应被消息框架反向塑形。

### 12.4 成熟基础设施库的使用边界

成熟库应主要用于以下层面：

- 消息 broker client
- 协议连接管理
- 序列化
- tracing / telemetry
- 集成测试基础设施

不应外包给第三方框架的层面包括：

- `DomainEvent` / `PublishedEvent` 的语义定义
- `ACL` 的翻译边界
- 本地投影模型
- context 间 published contract 的稳定边界
- 同步例外白名单机制

### 12.5 可观测性增强

为支撑事件驱动与最终一致性链路，目标态建议补充：

- `opentelemetry`
- `tracing-opentelemetry`
- `opentelemetry-otlp`

用于：

- 传播 `correlation_id` / `causation_id`
- 追踪 outbox 分发与 projector 链路
- 为同步例外与异步链路提供统一可观测性

### 12.6 测试基础设施增强

目标态建议补充：

- `testcontainers`
- `wiremock`

用于：

- Postgres / Redis / 未来消息系统的集成测试
- 同步 published interface 与外部 provider 的契约测试

### 12.7 外部消息总线的策略

目标态不预设外部 broker 为必选项。

如果未来系统边界继续增强，或需要真正的外部异步集成，再按以下优先级评估：

1. `async-nats`
   适合 Rust 友好、轻量、云原生、未来可拆分的路线
2. `rdkafka`
   适合组织基础设施本就以 Kafka 为核心的场景
3. `lapin`
   仅在既有 RabbitMQ 约束明确时考虑

在未触发明确外部消息需求前，不应提前引入 broker 复杂度。

### 12.8 暂不建议引入的方向

当前目标态不建议将以下类别作为核心骨架：

- ORM 替换型迁移，如将 `sqlx` 切换到 `diesel` 或 `sea-orm`
- 作业框架优先路线，如把 `apalis`、`tokio-cron-scheduler` 作为架构基础
- Rust DDD / CQRS / Event Sourcing 框架优先路线

这些方向不是不能用，而是当前并不应优先于：

- context 边界纯化
- published language 建立
- ACL 建立
- outbox / projector / projection 机制建立

## 13. 迁移路线

本设计定义的是目标态，因此迁移采用分阶段推进。

### 阶段一：重立边界骨架

目标：

- 建立新的 context 命名和 workspace 骨架
- 抽离平台公共抽象到独立 crate
- 建立 `published` / `integration` 的物理存在
- 建立最小架构测试框架

### 阶段二：重画核心业务边界

目标：

- 将现有 `menu` 重构为 `Catalog`
- 将现有 `authz` 重构为 `Access`
- 将现有 `order` 拆分为 `Ordering` 与 `Fulfillment`
- 将 `Identity` 压回认证主体本位

### 阶段三：建立事件主干与投影网络

目标：

- 为各 context 建立主干 published event
- 建立 ACL 与 integration read model
- 让关键读路径摆脱跨上下文数据库耦合

### 阶段四：完成纯化并收紧例外

目标：

- 建立同步例外白名单
- 清理残留耦合
- 建立投影重建、事件幂等与契约演进长期机制

## 14. 禁止项

为防止迁移过程产生新的中间态债务，明确禁止：

1. 将新 context 挂在旧 context 内部继续生长
2. 为赶进度让新 context 直接查询旧 context 表
3. 用 route DTO 继续充当 published language
4. 在未列入白名单时新增跨 context 同步依赖
5. 将平台公共抽象再次寄生到某个业务上下文之下

## 15. 成功标准

当目标态逐步落地后，后端应满足以下标准：

1. 后端 context map 稳定为六个能力型上下文
2. 平台公共抽象不再由业务上下文定义
3. 每个上下文具备清晰的 `domain / application / published / integration / infrastructure-*` 分层
4. schema ownership 与 context 一一对应
5. 跨上下文默认通过 published language、ACL、事件和本地投影协作
6. 同步例外被显式列出且受到自动化守卫
7. 架构测试能够阻止常见边界回退

## 16. 结语

本设计追求的不是“更整洁的现状”，而是“可长期维持纯度的目标架构”。  
它将当前后端从“已有模块化意识的单体”推进到“能力边界清晰、语言边界清晰、数据边界清晰、协作规则清晰的 DDD 模块化单体”。

后续所有实施计划都应以本设计为上位约束；若未来某项改动与本设计冲突，应先修订设计，再修订实现，而不是让实现默默偏离目标态。

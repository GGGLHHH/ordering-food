# Backend DDD Architecture Constitution

This document defines the target backend architecture for the `server` workspace.
It exists to prevent architectural drift. If implementation and this document diverge,
the correct action is to either:

1. bring the code back into compliance, or
2. update this document and the related architecture tests in the same change.

## Status

The backend target is:

- Modular monolith
- Strict DDD layering
- Bounded Contexts
- Hexagonal / Clean Architecture
- Published Language and ACL for cross-context collaboration
- Outbox-based asynchronous integration
- Background projection and async processing moved out of the HTTP entrypoint over time

This is the target state for all future backend evolution.

## Core Principles

### 1. Business model is the center

Domain rules must remain valid without:

- HTTP
- Axum
- SQLx
- Redis
- OpenAPI
- container runtime details

If removing those technologies breaks the core business model, the design is not clean enough.

### 2. Dependency direction always points inward

Allowed high-level direction:

`interface/adapters -> application -> domain`

Infrastructure depends on inner layers by implementing ports.
Inner layers must never depend on framework or storage details.

### 3. Bounded Contexts are business boundaries

Contexts are split by business capability, not by technical layer.
Current backend contexts include:

- `database`
- `identity`
- `organization`
- `access`
- `catalog`
- `ordering`
- `fulfillment`

These contexts may collaborate, but they must not collapse into a shared internal model.

### 4. Cross-context collaboration must be explicit

One context may collaborate with another only through:

- `published` contracts
- domain events
- an ACL inside `integration`

Direct imports of another context's `application` or `infrastructure` crates are forbidden.

### 5. The HTTP app is an outer adapter

`apps/api` is not a business layer.
It is the composition root and HTTP adapter for the modular monolith.

Its responsibilities are limited to:

- startup
- configuration
- context registration and assembly
- HTTP routing
- request extraction
- response mapping
- OpenAPI
- lifecycle hooks

It must not become a second application layer.

### 6. Explicit bootstrap is also an outer adapter

`apps/bootstrap` is not a business layer.
It is a dedicated outer entrypoint for explicit, opt-in bootstrap execution.

Its responsibilities are limited to:

- loading environment configuration
- wiring context runtimes needed for bootstrap
- invoking explicit seed/bootstrap flows
- reporting bootstrap failures

It must not become a hidden second HTTP app or a place for ad hoc business orchestration.

## Workspace Shape

Each business context should converge on this crate structure:

- `*-domain`
- `*-application`
- `*-published`
- `*-integration`
- `*-infrastructure-*`

Platform-level crates are allowed where they support the whole backend:

- `apps/api`
- `apps/bootstrap`
- `app-support`
- `bootstrap-core`
- `platform-kernel`
- `shared-kernel`
- `database-infrastructure-sqlx`

## Layer Responsibilities

### `domain`

Owns:

- entities
- value objects
- aggregates
- domain services
- domain events
- invariants
- specifications

Must not depend on:

- `axum`
- `sqlx`
- `redis`
- `serde` transport DTOs
- OpenAPI types
- HTTP concerns

A domain model must express business language, not transport or persistence language.

### `application`

Owns:

- use cases
- input and output models for use cases
- ports
- transaction boundaries
- authorization orchestration
- application services
- read model interfaces

May depend on:

- same-context `domain`
- same-context application ports
- same-context stable abstractions

Must not depend on:

- HTTP request or response types
- Axum extractors
- SQLx queries
- Redis clients
- concrete infrastructure implementations

Application logic coordinates business work. It does not become a thin wrapper over persistence.

### `infrastructure-*`

Owns:

- repository implementations
- query implementations
- token stores
- external client adapters
- outbox appenders
- persistence mapping
- integration with technical systems

May depend on:

- same-context `application`
- same-context `domain`
- technical libraries

Must not leak concrete implementation types into `apps/api` or other contexts.

### `published`

Owns only the stable collaboration surface exposed to other contexts:

- collaboration gateways
- collaboration DTOs
- published events
- collaboration error types

Must stay narrow.
It must not expose:

- internal repositories
- internal application services
- internal read models unless they are intentionally published language
- persistence details

### `integration`

Owns:

- runtime assembly for one context
- ACL translation for foreign published language
- infrastructure wiring
- translation between internal model and collaboration model

May depend on:

- same-context `domain`
- same-context `application`
- same-context `infrastructure-*`
- foreign `published`

This is the only place where those pieces should be assembled into a runnable context runtime.

### `apps/api`

Owns:

- route handlers
- request parsing
- auth extraction
- response DTO mapping
- error mapping
- OpenAPI declaration
- composition root

Must not depend directly on context-specific infrastructure crates.
Only composition code may assemble context runtimes.

### `apps/bootstrap`

Owns:

- explicit bootstrap command entrypoint
- bootstrap-specific runtime assembly
- bootstrap logging and failure reporting

Must not:

- host HTTP routes
- become a shared service layer for `apps/api`
- depend on `apps/api`

## Dependency Rules

### Allowed

- `apps/api -> *-application`
- `apps/api -> *-published`
- `apps/api -> *-integration`
- `apps/bootstrap -> *-published`
- `apps/bootstrap -> *-integration`
- `*-application -> same-context *-domain`
- `*-infrastructure-* -> same-context *-application`
- `*-infrastructure-* -> same-context *-domain`
- `*-integration -> same-context domain/application/infrastructure`
- `*-integration -> foreign *-published`
- `context A -> context B published`

### Forbidden

- `apps/api -> any context-specific infrastructure crate`
- `apps/bootstrap -> apps/api`
- `apps/bootstrap -> any context-specific infrastructure crate`
- `domain -> application`
- `domain -> infrastructure`
- `application -> axum/sqlx/redis`
- `application -> foreign application`
- `application -> foreign infrastructure`
- `infrastructure -> foreign application`
- `infrastructure -> foreign infrastructure`
- `route handlers -> SQLx`
- `route handlers -> context runtime internals`

## Cross-Context Collaboration Rules

### Published Language first

If one context needs information or a decision from another context, define the smallest stable
published contract that supports that collaboration.

### ACL in `integration`

If another context's published language does not match the receiving context's internal model,
the translation must live in the receiver's `integration` crate.

### No database reach-through

One context must not read another context's tables directly.
Cross-context reads must happen through:

- a published gateway, or
- a local projection updated by events

### Domain events for asynchronous collaboration

Asynchronous collaboration should use domain events persisted through outbox-style infrastructure.
Events are part of collaboration, not transport convenience.

## Aggregate and Transaction Rules

### Aggregates define consistency boundaries

Do not design transactions around "whatever tables are easiest to update together".
Design them around business invariants.

### One use case should prefer one aggregate boundary

If one use case consistently requires immediate multi-aggregate consistency,
that is a signal to revisit the model or move part of the workflow to asynchronous handling.

### Transactions belong to application boundaries

Transaction orchestration belongs to application or infrastructure supporting application ports.
It must not leak into route handlers.

## HTTP and Adapter Rules

### Route handlers stay thin

A route handler may:

- parse request data
- authenticate the caller
- call a use case or query service
- map the result to HTTP DTOs
- map application errors to API errors

A route handler must not:

- contain core business rules
- coordinate multiple repositories directly
- implement authorization policy internally
- write SQL
- manipulate infrastructure details

### API DTOs are outer-layer types

HTTP request and response DTOs belong to `apps/api`.
They must not be reused as domain entities.

### Error envelopes are stable contracts

The API must expose stable, transport-level error envelopes and must not leak internal exception
chains or infrastructure failure details to clients.

## Composition Root Rules

The composition root is where all concrete backend assembly happens.
Today that exists in two explicit outer entrypoints:

- `apps/api/src/composition/**` for HTTP runtime assembly
- `apps/bootstrap/src/**` for explicit bootstrap assembly

The composition root may:

- build context runtimes
- register migrations
- order bootstrapping
- resolve published capabilities
- wire lifecycle hooks

The composition root must not become a place for business rules.

## Background Jobs and Projection Rules

### Target architecture

Background processing should gradually move toward dedicated worker-style entrypoints.

Preferred shape:

- `apps/api` handles synchronous HTTP requests
- separate worker/projector apps handle projection, event consumption, and long-running async work

### Transitional rule

If a background job temporarily lives inside the HTTP process, it must still be modeled as
outer-layer lifecycle infrastructure, never as domain logic.

### Phase 3 event and projection path

长期目标态整改:

- integration runner 拉取 outbox 消息，并把 published events 交给下游处理链，而不是由 HTTP 请求路径直接承担跨上下文投影责任。
- projector 作为独立的后台执行单元消费事件并维护本地投影，避免同步读取外部上下文内部读模型。
- application handler 在事务中完成本地投影更新，确保消费后的状态落地和检查点推进具有清晰的一致性边界。
- 本地投影只服务于当前上下文自己的查询和协作需要，不得回退为跨上下文数据库直连。

## Migration and Seed Rules

### Migrations

Migrations may remain centrally orchestrated at the platform level for the modular monolith.

### Seed data

Development seed logic is allowed.
Production-critical seed/bootstrap behavior should move toward explicit bootstrap commands or jobs.
Seed logic must not silently become permanent business behavior in the main request path.
Default API startup must not auto-seed business contexts.
If temporary development bootstrap is needed, it must be executed through an explicit bootstrap command
or job, not by toggling HTTP startup behavior.
The current explicit bootstrap command is `cargo run -p ordering-food-bootstrap`.

Migrations may remain automatic at startup.
Business seed/bootstrap must remain opt-in until it is fully extracted into explicit commands or jobs.

## `shared-kernel` Rules

`shared-kernel` must stay minimal.
It may contain only truly shared primitives with strong semantic stability.

It must not become:

- a dump for convenience utilities
- a hidden cross-context dependency shortcut
- a place for business concepts owned by one context

When in doubt, keep concepts inside the owning context.

## Architecture Testing Rules

Architecture boundaries must be protected by tests.
The codebase must continue to maintain architecture tests that verify at least:

- route modules do not depend on infrastructure crates
- composition is the only place that assembles runtimes
- contexts depend on foreign `published`, not foreign `application` or `infrastructure`
- legacy contexts do not creep back in through dependencies
- `apps/api` does not store business modules in global app state

Every change that modifies architecture boundaries should update or add architecture tests.

## Change Control Checklist

Any backend PR touching architecture-sensitive code must answer these questions:

1. Which bounded context owns this rule?
2. Does this change introduce a new cross-context dependency?
3. If yes, is that dependency expressed through `published` or ACL?
4. Did any inner layer gain a framework or storage dependency?
5. Did any route handler become responsible for business decisions?
6. Should this change add or update architecture tests?
7. Does this document still describe reality after the change?

If the answer to question 7 is "no", this document must be updated in the same PR.

## Non-Negotiable Rules

The following are considered architectural violations:

- importing context-specific infrastructure directly into route handlers
- importing another context's internal application service
- importing another context's infrastructure implementation
- placing business invariants in HTTP handlers
- treating ORM or query models as domain entities
- using `shared-kernel` to bypass context ownership
- leaking internal exceptions or storage details through API contracts

## Target Outcome

The backend is considered aligned with this constitution when:

- the domain remains framework-independent
- application services orchestrate but do not persist directly
- infrastructure stays behind ports
- contexts collaborate only through explicit published language or events
- the HTTP app remains an adapter, not a business layer
- architecture tests continuously enforce those boundaries

This document is intentionally strict. Convenience is not a sufficient reason to violate it.

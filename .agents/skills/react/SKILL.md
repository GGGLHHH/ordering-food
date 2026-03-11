---
name: react
description: React 19.2 component model, hooks, Actions/forms, concurrent rendering, SSR hydration, and React Compiler usage. Use when building or refactoring modern React apps with function components and TypeScript.
metadata:
  author: Anthony Fu
  version: "2026.2.13"
  source: Generated from https://github.com/reactjs/react.dev, scripts located at https://github.com/GGGLHHH/skills
---

# React

> The skill is based on React 19.2 docs (react.dev latest), generated at 2026-02-13.

React 19.2 centers around pure components, Hook-based state, Action-driven forms, concurrent rendering primitives, and stronger compile-time optimization through React Compiler.

## Preferences

- Prefer TypeScript and function components.
- Keep Effects for external synchronization only.
- Use Action patterns for form submissions and optimistic updates.
- Treat React Compiler as default optimization, add manual memoization only for measured hotspots.

## Core

| Topic | Description | Reference |
|-------|-------------|-----------|
| State Model | State snapshots, update queues, reducers, and context composition | [core-state-model](references/core-state-model.md) |
| Effects & Effect Events | Correct dependency modeling, cleanup, and `useEffectEvent` patterns | [core-effects-and-events](references/core-effects-and-events.md) |

## Features

| Topic | Description | Reference |
|-------|-------------|-----------|
| Actions & Forms | `<form action>`, `useActionState`, `useFormStatus`, `useOptimistic` | [features-actions-and-forms](references/features-actions-and-forms.md) |
| Transitions & Suspense | `useTransition`, `startTransition`, `useDeferredValue`, `Suspense` | [features-transitions-and-suspense](references/features-transitions-and-suspense.md) |
| Async Resources | `use(resource)` and server-only `cache()` patterns | [features-async-resources](references/features-async-resources.md) |

## Best Practices

| Topic | Description | Reference |
|-------|-------------|-----------|
| Purity & Hook Rules | Hook call constraints, immutable data flow, and "you might not need an effect" | [best-practices-purity-and-hooks-rules](references/best-practices-purity-and-hooks-rules.md) |

## Advanced

| Topic | Description | Reference |
|-------|-------------|-----------|
| SSR & Hydration | Streaming HTML, hydration constraints, and mismatch prevention | [advanced-ssr-and-hydration](references/advanced-ssr-and-hydration.md) |
| React Compiler | Compiler modes, directives, rollout strategy, and safety controls | [advanced-react-compiler](references/advanced-react-compiler.md) |

## Quick Reference

### Core Packages

```bash
npm i react react-dom
npm i -D @types/react @types/react-dom
```

### Root APIs

```ts
import { createRoot, hydrateRoot } from 'react-dom/client'
import { renderToPipeableStream } from 'react-dom/server'
```

### Form + Action Shape

```tsx
const [state, formAction, isPending] = useActionState(action, initialState)

return (
  <form action={formAction}>
    <button disabled={isPending}>Submit</button>
  </form>
)
```

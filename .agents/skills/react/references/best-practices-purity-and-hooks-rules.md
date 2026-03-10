---
name: react-purity-and-hooks-rules
description: Rules of Hooks, render purity, immutable updates, and Effect minimization patterns
---

# Purity and Hook Rules

React correctness and performance both depend on predictable render behavior.

## Rules of Hooks (Non-Negotiable)

- Call Hooks only at the top level.
- Call Hooks only from React components or custom Hooks.
- Never call Hooks in loops, conditions, event handlers, or `try/catch` blocks.

```tsx
// good
function Profile() {
  const [name] = useState('')
  const theme = useContext(ThemeContext)
  return <h1 className={theme}>{name}</h1>
}
```

## Keep Render Pure

Render must be idempotent for identical inputs.

Avoid in render:

- Mutating values outside component scope.
- Calling browser side effects (e.g. `document.title = ...`).
- Non-deterministic calculations that must be synchronized over time.

```tsx
// avoid
function Page({ title }: { title: string }) {
  document.title = title
  return <h1>{title}</h1>
}
```

Use Effect for external synchronization instead:

```tsx
function Page({ title }: { title: string }) {
  useEffect(() => {
    document.title = title
  }, [title])
  return <h1>{title}</h1>
}
```

## Prefer Derivation Over Effect-Driven State

If value can be computed from props/state, derive during render.

```tsx
const fullName = `${firstName} ${lastName}`
```

Do not mirror it into state with an Effect.

## Immutable Data Flow

Never mutate props or state directly.

```tsx
setItems(prev => prev.map(item => item.id === id ? { ...item, done: true } : item))
```

Local mutation of values created in the same render is acceptable when not leaked outside.

## Linting Baseline

Enable `eslint-plugin-react-hooks` and treat violations as build blockers.

<!--
Source references:
- https://react.dev/reference/rules/rules-of-hooks
- https://react.dev/reference/rules/components-and-hooks-must-be-pure
- https://react.dev/learn/you-might-not-need-an-effect
- https://react.dev/learn/removing-effect-dependencies
-->

---
name: react-compiler
description: React Compiler setup, modes, directives, and rollout controls
---

# React Compiler

React Compiler provides automatic memoization and render optimization at build time.

## Minimal Setup

```js
// babel.config.js
module.exports = {
  plugins: ['babel-plugin-react-compiler'],
}
```

For most React 19 projects, defaults are enough.

## Compilation Modes

```js
// babel.config.js
module.exports = {
  plugins: [['babel-plugin-react-compiler', {
    compilationMode: 'infer', // infer | annotation | all
  }]],
}
```

- `infer` (default): compile inferred components/hooks by naming conventions.
- `annotation`: compile only functions explicitly marked with `"use memo"`.
- `all`: compile broadly (use carefully for gradual rollout).

## Directives

### Opt-in with `"use memo"`

```ts
function ProductCard() {
  'use memo'
  // ...
}
```

Useful when running `annotation` mode.

### Opt-out with `"use no memo"`

```ts
function ThirdPartyWrapper() {
  'use no memo'
  // temporary escape hatch
}
```

Use only for debugging/compatibility, then remove.

## Compatibility and Safety Knobs

```js
{
  target: '18',
  panicThreshold: 'none',
  gating: {
    source: 'feature-flags',
    importSpecifierName: 'isCompilerEnabled',
  },
}
```

- `target`: set for React 17/18 compatibility (plus `react-compiler-runtime`).
- `panicThreshold`: control fail-fast vs skip behavior.
- `gating`: feature-flag optimized output for incremental rollouts.

## Adoption Strategy

1. Start with default `infer` and fix rule violations.
2. Track compile successes/failures via `logger`.
3. Use `"use no memo"` only as short-lived exceptions.

<!--
Source references:
- https://react.dev/reference/react-compiler/configuration
- https://react.dev/reference/react-compiler/compilationMode
- https://react.dev/reference/react-compiler/directives/use-memo
- https://react.dev/reference/react-compiler/directives/use-no-memo
- https://react.dev/reference/react-compiler/target
- https://react.dev/reference/react-compiler/panicThreshold
- https://react.dev/reference/react-compiler/gating
-->

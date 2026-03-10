---
name: react-state-model
description: State snapshots, update queues, reducers, and context patterns in React 19
---

# State Model

Use React state as immutable snapshots. Model updates as transitions from one snapshot to the next.

## Prefer Functional Updates for Dependent Writes

When next state depends on previous state, always use updater functions.

```tsx
import { useState } from 'react'

export function Counter() {
  const [count, setCount] = useState(0)

  function incrementThreeTimes() {
    setCount(c => c + 1)
    setCount(c => c + 1)
    setCount(c => c + 1)
  }

  return <button onClick={incrementThreeTimes}>{count}</button>
}
```

## Use Reducers for Multi-Action State

Reducers are better than multiple interdependent `useState` calls when updates are action-based.

```tsx
import { useReducer } from 'react'

type State = { items: string[]; filter: 'all' | 'open' | 'done' }
type Action =
  | { type: 'add'; text: string }
  | { type: 'setFilter'; filter: State['filter'] }

function reducer(state: State, action: Action): State {
  switch (action.type) {
    case 'add':
      return { ...state, items: [...state.items, action.text] }
    case 'setFilter':
      return { ...state, filter: action.filter }
  }
}

export function TodoState() {
  const [state, dispatch] = useReducer(reducer, { items: [], filter: 'all' })
  return (
    <>
      <button onClick={() => dispatch({ type: 'add', text: 'Ship feature' })}>Add</button>
      <button onClick={() => dispatch({ type: 'setFilter', filter: 'open' })}>Open</button>
      <pre>{JSON.stringify(state, null, 2)}</pre>
    </>
  )
}
```

## Context as a Read API, Not a Mutation API

Keep mutation APIs close to ownership boundaries. Pass stable action functions through context.

```tsx
import { createContext, useContext, useMemo, useState } from 'react'

type Theme = 'light' | 'dark'
type ThemeContextValue = { theme: Theme; setTheme: (t: Theme) => void }

const ThemeContext = createContext<ThemeContextValue | null>(null)

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [theme, setTheme] = useState<Theme>('light')
  const value = useMemo(() => ({ theme, setTheme }), [theme])
  return <ThemeContext value={value}>{children}</ThemeContext>
}

export function useTheme() {
  const ctx = useContext(ThemeContext)
  if (!ctx)
    throw new Error('ThemeContext missing')
  return ctx
}
```

## Decision Heuristic

- Use `useState` for local, independent fields.
- Use `useReducer` for state with explicit action semantics.
- Use context to avoid prop drilling, not as global mutable storage by default.

<!--
Source references:
- https://react.dev/reference/react/useState
- https://react.dev/reference/react/useReducer
- https://react.dev/reference/react/useContext
- https://react.dev/learn/managing-state
-->

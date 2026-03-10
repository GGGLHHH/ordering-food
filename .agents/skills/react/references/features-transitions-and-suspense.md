---
name: react-transitions-and-suspense
description: Concurrent rendering primitives with useTransition, startTransition, useDeferredValue, and Suspense
---

# Transitions and Suspense

Use transition APIs to keep urgent updates (typing/clicking) responsive while background updates are processed.

## Mark Non-Urgent Updates with `useTransition`

```tsx
import { useState, useTransition } from 'react'

export function TabView() {
  const [tab, setTab] = useState<'overview' | 'logs'>('overview')
  const [isPending, startTransition] = useTransition()

  function selectTab(next: 'overview' | 'logs') {
    startTransition(() => {
      setTab(next)
    })
  }

  return (
    <>
      <button onClick={() => selectTab('overview')}>Overview</button>
      <button onClick={() => selectTab('logs')}>Logs</button>
      {isPending && <p>Loading...</p>}
      <Panel tab={tab} />
    </>
  )
}
```

## For External Libraries, Use Standalone `startTransition`

If you are outside a component (e.g. data layer), import `startTransition` from `react`.

```ts
import { startTransition } from 'react'

startTransition(() => {
  store.setState(nextState)
})
```

## Defer Expensive Render Paths with `useDeferredValue`

```tsx
import { useDeferredValue, useMemo, useState } from 'react'

function SearchList({ items }: { items: string[] }) {
  const [query, setQuery] = useState('')
  const deferredQuery = useDeferredValue(query)

  const filtered = useMemo(
    () => items.filter(i => i.toLowerCase().includes(deferredQuery.toLowerCase())),
    [items, deferredQuery],
  )

  return (
    <>
      <input value={query} onChange={e => setQuery(e.target.value)} />
      <Results items={filtered} dimmed={query !== deferredQuery} />
    </>
  )
}
```

## Use Suspense Boundaries Intentionally

- Put boundaries around async subtrees, not the whole app shell.
- Use nested boundaries for progressive reveal.
- Keep fallback UI lightweight and stable.

```tsx
<Suspense fallback={<FeedSkeleton />}>
  <Feed />
</Suspense>
```

<!--
Source references:
- https://react.dev/reference/react/useTransition
- https://react.dev/reference/react/startTransition
- https://react.dev/reference/react/useDeferredValue
- https://react.dev/reference/react/Suspense
-->

---
name: react-async-resources
description: Async resource handling with use(resource) and server-side cache() in React 19
---

# Async Resources

React 19 includes resource-style APIs for promise/context reading (`use`) and server-side memoized fetch/computation (`cache`).

## `use(resource)` for Promises and Context

```tsx
import { Suspense, use } from 'react'

function Message({ messagePromise }: { messagePromise: Promise<string> }) {
  const message = use(messagePromise)
  return <p>{message}</p>
}

export function MessagePanel({ promise }: { promise: Promise<string> }) {
  return (
    <Suspense fallback={<p>Loading...</p>}>
      <Message messagePromise={promise} />
    </Suspense>
  )
}
```

Key behavior:

- With a pending Promise, component suspends and nearest Suspense fallback is shown.
- With rejected Promise, nearest Error Boundary handles it.
- Unlike Hooks, `use` can be called in conditionals and loops.

## Prefer Server-Created Promises

For better stability, create promises in Server Components (or framework loaders) and pass them to client boundaries, instead of recreating promises in client renders.

## `cache(fn)` for Server Components

`cache` memoizes a function per server request.

```ts
import { cache } from 'react'

export const getUser = cache(async (id: string) => {
  const res = await fetch(`https://api.example.com/users/${id}`)
  return res.json()
})
```

Important caveats:

- Server-only API.
- Cache is invalidated between requests.
- Re-calling `cache(fn)` creates a different memoized function with a different cache.

## Recommended Layering

- Use framework data APIs for routing-level data.
- Use `use` + Suspense for resource reading in component trees.
- Use `cache` for deduplicated server fetch/compute shared by multiple components.

<!--
Source references:
- https://react.dev/reference/react/use
- https://react.dev/reference/react/cache
- https://react.dev/reference/react/Suspense
-->

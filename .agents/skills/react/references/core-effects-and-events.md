---
name: react-effects-and-events
description: Effect synchronization, dependency hygiene, and useEffectEvent patterns
---

# Effects and Effect Events

Effects are for synchronization with external systems (network, timers, subscriptions, DOM APIs).

## Treat Effects as Start/Stop Synchronization

```tsx
import { useEffect } from 'react'

export function ChatRoom({ roomId }: { roomId: string }) {
  useEffect(() => {
    const connection = createConnection(roomId)
    connection.connect()
    return () => connection.disconnect()
  }, [roomId])

  return null
}
```

Dependency rule: every reactive value read in Effect setup/cleanup must appear in the dependency list.

## Avoid Derived State in Effects

Bad: deriving render data through extra state + Effect.

```tsx
// avoid
const [fullName, setFullName] = useState('')
useEffect(() => {
  setFullName(`${first} ${last}`)
}, [first, last])
```

Good: derive directly during render.

```tsx
const fullName = `${first} ${last}`
```

## Use `useEffectEvent` for Non-Reactive Logic Inside Effects

When Effect logic needs latest values but should not re-subscribe, extract an Effect Event.

```tsx
import { useEffect, useEffectEvent } from 'react'

export function Room({ roomId, theme }: { roomId: string; theme: 'light' | 'dark' }) {
  const onConnected = useEffectEvent(() => {
    showNotification('Connected', theme)
  })

  useEffect(() => {
    const connection = createConnection(roomId)
    connection.on('connected', onConnected)
    connection.connect()
    return () => connection.disconnect()
  }, [roomId])

  return null
}
```

## Prefer Event Handlers for User-Driven Side Effects

If logic is caused by a click/submit, keep it in the handler instead of waiting for an Effect.

```tsx
async function handleBuy() {
  await post('/api/buy')
  toast('Purchased')
}
```

## Common Smells

- Effect only writes local state derived from props/state.
- Effect dependency suppression (`// eslint-disable-next-line react-hooks/exhaustive-deps`).
- Recreating subscriptions on every render due unstable object/function dependencies.

<!--
Source references:
- https://react.dev/reference/react/useEffect
- https://react.dev/reference/react/useEffectEvent
- https://react.dev/learn/you-might-not-need-an-effect
- https://react.dev/learn/removing-effect-dependencies
-->

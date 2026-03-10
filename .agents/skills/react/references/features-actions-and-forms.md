---
name: react-actions-and-forms
description: React 19 form action patterns with useActionState, useFormStatus, and useOptimistic
---

# Actions and Forms

React 19 lets forms call functions directly via `<form action={fn}>` and coordinate pending, result, and optimistic state with dedicated APIs.

## Client Action with `<form action>`

```tsx
export function SearchForm() {
  async function search(formData: FormData) {
    const query = String(formData.get('query') ?? '')
    await runSearch(query)
  }

  return (
    <form action={search}>
      <input name="query" />
      <button type="submit">Search</button>
    </form>
  )
}
```

React resets uncontrolled form fields after successful function actions.

## Track Returned State with `useActionState`

```tsx
import { useActionState } from 'react'

type Result = { ok: boolean; message: string }

async function submit(prev: Result | null, formData: FormData): Promise<Result> {
  const email = String(formData.get('email') ?? '')
  if (!email.includes('@'))
    return { ok: false, message: 'Invalid email' }
  await saveEmail(email)
  return { ok: true, message: 'Saved' }
}

export function NewsletterForm() {
  const [state, formAction, isPending] = useActionState(submit, null)

  return (
    <form action={formAction}>
      <input name="email" />
      <button disabled={isPending}>Join</button>
      <p>{state?.message}</p>
    </form>
  )
}
```

## Read Pending/Data with `useFormStatus`

Call inside a component rendered inside the corresponding form.

```tsx
import { useFormStatus } from 'react-dom'

function SubmitButton() {
  const { pending } = useFormStatus()
  return <button type="submit" disabled={pending}>{pending ? 'Submitting...' : 'Submit'}</button>
}
```

## Optimistic UI with `useOptimistic`

`useOptimistic` is designed for Action/Transition workflows.

```tsx
import { useOptimistic } from 'react'

const [optimisticItems, addOptimisticItem] = useOptimistic(items, (state, next: string) => [
  ...state,
  { text: next, sending: true },
])
```

Apply optimistic update before awaiting network I/O, then reconcile with real state.

## Practical Pattern

- Submission result: `useActionState`
- Submission pending flag: `useFormStatus` in nested button
- Immediate UX response: `useOptimistic`

<!--
Source references:
- https://react.dev/reference/react-dom/components/form
- https://react.dev/reference/react/useActionState
- https://react.dev/reference/react-dom/hooks/useFormStatus
- https://react.dev/reference/react/useOptimistic
-->

---
name: tailwindcss-dynamic-class-strategies
description: Reliable runtime conditional styling patterns that keep Tailwind output deterministic
---

# Dynamic Class Strategies

Tailwind only generates utilities it can detect statically. Build runtime style logic around complete class names.

## Use Mapping Tables Instead of String Assembly

```tsx
type Intent = "success" | "warning" | "danger"

const intentClass: Record<Intent, string> = {
  success: "bg-emerald-600 text-white hover:bg-emerald-500",
  warning: "bg-amber-500 text-black hover:bg-amber-400",
  danger: "bg-rose-600 text-white hover:bg-rose-500",
}

export function AlertButton({ intent }: { intent: Intent }) {
  return <button className={`rounded-md px-3 py-2 ${intentClass[intent]}`}>Action</button>
}
```

## Compose with Class Utilities

```tsx
import clsx from "clsx"

const className = clsx(
  "rounded-md px-3 py-2",
  active ? "bg-sky-600 text-white" : "bg-zinc-100 text-zinc-900",
)
```

## Safelist Last Resort

If a class must be generated from runtime content, safelist it intentionally with `@source inline()` instead of implicit string concatenation.

## Team Rule

- Allow only static utility strings in shared components.
- Put variant decisions in typed maps near component props.
- Review PRs for patterns like `` `bg-${x}-500` `` and replace them.

<!--
Source references:
- https://tailwindcss.com/docs/detecting-classes-in-source-files
-->

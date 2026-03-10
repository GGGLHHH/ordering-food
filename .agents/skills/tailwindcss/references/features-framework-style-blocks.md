---
name: tailwindcss-framework-style-blocks
description: Use Tailwind utilities inside component-scoped style blocks with @reference
---

# Framework Style Blocks

When using Vue, Svelte, or CSS Modules, scoped style blocks do not automatically know theme variables and utilities. Use `@reference`.

## Vue SFC Example

```vue
<template>
  <button class="btn-primary">Save</button>
</template>

<style scoped>
@reference "../../app.css";

.btn-primary {
  @apply rounded-md bg-brand-600 px-4 py-2 text-white hover:bg-brand-500;
}
</style>
```

## CSS Modules Example

```css
/* button.module.css */
@reference "../app.css";

.primary {
  @apply inline-flex items-center gap-2 rounded-md bg-zinc-900 px-3 py-2 text-white;
}
```

```tsx
import s from "./button.module.css"

export function Button() {
  return <button className={s.primary}>Run</button>
}
```

## Guideline

- Prefer inline utilities in templates for most components.
- Use style blocks + `@apply` for heavily reused semantic classes.
- Keep one canonical `app.css` entry and reference it consistently.

<!--
Source references:
- https://tailwindcss.com/docs/functions-and-directives
- https://tailwindcss.com/docs/compatibility
-->

---
name: tailwindcss-variants-responsive-dark-mode
description: Compose interactive, responsive, and dark-mode styles using Tailwind variants
---

# Variants, Responsive, and Dark Mode

Tailwind variants are state/query prefixes that compose directly in class strings.

## State Variants

```html
<button class="bg-sky-600 text-white hover:bg-sky-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-500 disabled:opacity-40">
  Save
</button>
```

Use variant prefixes instead of separate CSS selectors.

## Responsive Variants

```html
<div class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-4">
  ...
</div>
```

Design mobile-first, then add `sm:`, `md:`, `lg:`, `xl:`, `2xl:` overrides.

## Dark Mode

By default, `dark:` uses the user OS setting. For manual toggles, define a custom variant and set class/data attribute.

```css
@import "tailwindcss";
@custom-variant dark (&:where(.dark, .dark *));
```

```html
<html class="dark">
  <body class="bg-white text-zinc-900 dark:bg-zinc-900 dark:text-zinc-100">
    ...
  </body>
</html>
```

## Data/ARIA/Container Variants

```html
<div data-state="open" class="data-[state=open]:block hidden"></div>
<button aria-pressed="true" class="aria-pressed:bg-sky-600"></button>
<section class="@container">
  <div class="@md:grid @md:grid-cols-2">...</div>
</section>
```

Use these variants to avoid custom selectors for component state and adaptive layouts.

<!--
Source references:
- https://tailwindcss.com/docs/hover-focus-and-other-states
- https://tailwindcss.com/docs/breakpoints
- https://tailwindcss.com/docs/dark-mode
-->

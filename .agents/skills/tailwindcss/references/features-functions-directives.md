---
name: tailwindcss-functions-and-directives
description: Use Tailwind v4 directives to define custom utilities and integrate legacy config
---

# Functions and Directives

Tailwind v4 exposes CSS directives for extending behavior without leaving stylesheets.

## `@utility` for Project-Level Utilities

```css
@import "tailwindcss";

@utility btn {
  border-radius: --spacing(2);
  padding-inline: --spacing(4);
  padding-block: --spacing(2);
  font-weight: 600;
}
```

Use in templates as `class="btn"`.

## `@variant` and `@custom-variant`

```css
@import "tailwindcss";

@custom-variant theme-midnight (&:where([data-theme="midnight"] *));

.card {
  @variant hover {
    box-shadow: 0 10px 25px rgb(0 0 0 / 0.2);
  }

  @variant theme-midnight {
    background: oklch(0.24 0.02 265);
  }
}
```

## `@apply` for Small Reusable CSS Blocks

```css
.card-title {
  @apply text-lg font-semibold tracking-tight;
}
```

Use sparingly; prefer utility classes directly in markup first.

## `@config` and `@plugin` for Transitional Migrations

```css
@import "tailwindcss";
@config "../../tailwind.config.js";
@plugin "@tailwindcss/typography";
```

Useful when migrating existing v3 projects incrementally.

<!--
Source references:
- https://tailwindcss.com/docs/functions-and-directives
-->

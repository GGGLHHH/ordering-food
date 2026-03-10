---
name: tailwindcss-theme-variables
description: Define and consume Tailwind design tokens with @theme in v4
---

# Theme Variables

Tailwind v4 promotes CSS-first theming via `@theme`. Use it as the source of truth for design tokens.

## Define Tokens

```css
@import "tailwindcss";

@theme {
  --color-brand-50: oklch(0.97 0.02 250);
  --color-brand-500: oklch(0.62 0.19 252);
  --color-brand-700: oklch(0.48 0.16 252);

  --font-sans: "Inter", "sans-serif";
  --spacing-18: 4.5rem;
  --radius-card: 1rem;
}
```

These tokens become available in utilities such as `bg-brand-500`, `font-sans`, `p-18`, and `rounded-card`.

## Override Tokens by Scope

```css
:root {
  --color-brand-500: oklch(0.62 0.19 252);
}

[data-theme="enterprise"] {
  --color-brand-500: oklch(0.56 0.23 267);
}
```

Use this for tenant branding and theme switching without changing class names.

## Add Custom Breakpoint Tokens

```css
@theme {
  --breakpoint-3xl: 120rem;
}
```

Now `3xl:` becomes available as a responsive variant prefix.

## Guideline

- Keep semantic token names (`brand`, `surface`, `danger`) instead of hard-coded intent in components.
- Keep tokens centralized; avoid redefining `@theme` in many files.

<!--
Source references:
- https://tailwindcss.com/docs/theme
- https://tailwindcss.com/docs/customizing-spacing
-->

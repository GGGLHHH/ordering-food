---
name: tailwindcss-class-detection
description: How Tailwind v4 scans class usage and how to include additional sources safely
---

# Class Detection

Tailwind generates CSS by scanning files for complete class names. If a class is not detectable, it will not be emitted.

## Keep Class Names Static

Good:

```tsx
const variants = {
  primary: "bg-blue-600 text-white hover:bg-blue-500",
  neutral: "bg-zinc-100 text-zinc-900 hover:bg-zinc-200",
}

<button className={variants[intent]} />
```

Avoid building fragments like `bg-${color}-600`.

## Register Extra Sources with `@source`

If classes live outside default scan paths (shared package, external UI lib), add explicit sources.

```css
@import "tailwindcss";
@source "../node_modules/@my-company/ui-lib";
```

## Safelist Specific Utilities with `@source inline()`

For runtime-only combinations, safelist exact patterns.

```css
@import "tailwindcss";
@source inline("{hover:,focus:,}bg-brand-{50,{100..900..100},950}");
```

## Exclude Unwanted Paths with `@source not`

```css
@import "tailwindcss";
@source not "../src/legacy";
```

Use this to reduce build size when scanning monorepos.

<!--
Source references:
- https://tailwindcss.com/docs/detecting-classes-in-source-files
-->

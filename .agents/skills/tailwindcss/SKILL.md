---
name: tailwindcss
description: Tailwind CSS v4 utility-first workflow covering setup, source detection, theme variables, variants, directives, and migration guidance. Use when building or refactoring UI styles in React, Vue, Nuxt, Vite, or plain HTML projects.
metadata:
  author: Anthony Fu
  version: "2026.2.14"
  source: Generated from https://github.com/tailwindlabs/tailwindcss.com, scripts located at https://github.com/antfu/skills
---

# Tailwind CSS

> The skill is based on Tailwind CSS v4.1 docs, generated at 2026-02-14.

Tailwind CSS v4 is CSS-first and utility-driven. Use it to style interfaces through composable class tokens, theme variables, and variant prefixes instead of large hand-written stylesheet hierarchies.

## Preferences

- Prefer v4 default setup (`@import "tailwindcss"`).
- Keep class names statically detectable by Tailwind.
- Model design tokens with `@theme` variables.
- Use utility variants before writing custom CSS.

## Core

| Topic | Description | Reference |
|-------|-------------|-----------|
| Installation & Toolchain | Vite/PostCSS/CLI setup patterns and production build workflow | [core-installation-and-toolchain](references/core-installation-and-toolchain.md) |
| Class Detection | How Tailwind scans source files and how to include/safelist classes | [core-class-detection](references/core-class-detection.md) |
| Theme Variables | Define colors, spacing, fonts, and breakpoints with `@theme` | [core-theme-variables](references/core-theme-variables.md) |

## Features

| Topic | Description | Reference |
|-------|-------------|-----------|
| Variants & Responsive | `hover:`, `focus:`, `dark:`, breakpoints, container query variants | [features-variants-responsive-dark-mode](references/features-variants-responsive-dark-mode.md) |
| Directives & Custom Utilities | `@utility`, `@variant`, `@custom-variant`, `@apply`, `@source`, `@reference` | [features-functions-directives](references/features-functions-directives.md) |
| Framework Style Blocks | Using Tailwind utilities inside Vue/Svelte/CSS Modules with `@reference` | [features-framework-style-blocks](references/features-framework-style-blocks.md) |

## Best Practices

| Topic | Description | Reference |
|-------|-------------|-----------|
| Dynamic Class Strategies | Safe patterns for conditional styling without breaking class detection | [best-practices-dynamic-class-strategies](references/best-practices-dynamic-class-strategies.md) |

## Advanced

| Topic | Description | Reference |
|-------|-------------|-----------|
| Upgrade v3 to v4 | Package/API changes, config migration, and compatibility mode choices | [advanced-upgrade-v3-to-v4](references/advanced-upgrade-v3-to-v4.md) |
| Compatibility | Browser support, preprocessor constraints, and architecture decisions | [advanced-compatibility](references/advanced-compatibility.md) |

## Quick Reference

### Core Setup

```css
/* app.css */
@import "tailwindcss";
```

### PostCSS Plugin

```js
// postcss.config.mjs
export default {
  plugins: {
    "@tailwindcss/postcss": {},
  },
}
```

### Vite Plugin

```ts
// vite.config.ts
import { defineConfig } from "vite"
import tailwindcss from "@tailwindcss/vite"

export default defineConfig({
  plugins: [tailwindcss()],
})
```

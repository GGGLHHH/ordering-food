---
name: tailwindcss-installation-and-toolchain
description: Tailwind CSS v4 setup patterns for Vite, PostCSS, and CLI builds
---

# Installation and Toolchain

Tailwind CSS v4 can run through three common pipelines. Pick one and keep setup minimal.

## Option A: Vite Plugin (Recommended for Vite Apps)

```bash
npm i tailwindcss @tailwindcss/vite
```

```ts
// vite.config.ts
import { defineConfig } from "vite"
import tailwindcss from "@tailwindcss/vite"

export default defineConfig({
  plugins: [tailwindcss()],
})
```

```css
/* src/style.css */
@import "tailwindcss";
```

Why: best DX when app already uses Vite.

## Option B: PostCSS Plugin (Framework-Agnostic)

```bash
npm i tailwindcss @tailwindcss/postcss postcss
```

```js
// postcss.config.mjs
export default {
  plugins: {
    "@tailwindcss/postcss": {},
  },
}
```

```css
@import "tailwindcss";
```

Why: works in Next.js and other PostCSS-based pipelines.

## Option C: Tailwind CLI (Simple Static Sites)

```bash
npm i tailwindcss @tailwindcss/cli
npx @tailwindcss/cli -i ./src/input.css -o ./dist/output.css --watch
```

```css
/* src/input.css */
@import "tailwindcss";
```

Why: no bundler required.

## Practical Notes

- Keep one root CSS entry with `@import "tailwindcss";`.
- Import generated CSS before component-level overrides.
- If runtime uses multiple apps/packages, ensure each package has a deterministic CSS entry.

<!--
Source references:
- https://tailwindcss.com/docs/installation/using-vite
- https://tailwindcss.com/docs/installation/using-postcss
- https://tailwindcss.com/docs/installation/tailwind-cli
-->

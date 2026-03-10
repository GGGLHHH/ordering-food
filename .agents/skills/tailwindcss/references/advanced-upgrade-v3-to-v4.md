---
name: tailwindcss-upgrade-v3-to-v4
description: Tailwind v3 to v4 migration checklist and rollout strategy
---

# Upgrade v3 to v4

Tailwind v4 is a major architecture change. Migrate in explicit steps.

## Step 1: Use Upgrade Tool on an Isolated Branch

```bash
npx @tailwindcss/upgrade
```

Then review generated diffs manually.

## Step 2: Update Import Style

Old:

```css
@tailwind base;
@tailwind components;
@tailwind utilities;
```

New:

```css
@import "tailwindcss";
```

## Step 3: Update Build Integrations

- PostCSS: move from `tailwindcss` plugin to `@tailwindcss/postcss`.
- CLI: use `@tailwindcss/cli` package.
- Vite: prefer `@tailwindcss/vite` plugin.

## Step 4: Migrate Configuration Thoughtfully

- Keep legacy config temporarily with `@config` when needed.
- Migrate design tokens to `@theme` gradually.
- Validate custom plugin behavior and directives (`@plugin`, `@utility`).

## Step 5: Verify Runtime and Visual Regressions

- Snapshot core pages and shared components.
- Check dark mode, data variants, and responsive behavior.
- Confirm safelist patterns still produce expected utilities.

## Rollout Strategy

- Migrate design-system package first.
- Upgrade downstream apps one by one.
- Keep short-lived compatibility shims; remove them after stabilization.

<!--
Source references:
- https://tailwindcss.com/docs/upgrade-guide
- https://tailwindcss.com/docs/functions-and-directives
-->

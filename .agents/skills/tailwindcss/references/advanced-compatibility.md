---
name: tailwindcss-compatibility
description: Browser and tooling compatibility constraints for Tailwind CSS v4
---

# Compatibility

Use this checklist before adopting Tailwind v4 in existing systems.

## Browser Baseline

Tailwind v4 targets modern browsers (for example Chrome 111+, Safari 16.4+, Firefox 128+).

Implication:

- Legacy browser support requires either staying on v3.4 or splitting styling strategy.
- Verify product browser matrix before migration.

## Preprocessors (Sass/Less/Stylus)

Tailwind v4 is intended to be the CSS build tool itself.

- Avoid stacking Sass/Less/Stylus on top of Tailwind by default.
- Use native CSS variables, nesting, and math/color functions where needed.

## CSS Modules and Scoped Styles

Tailwind can coexist with CSS Modules, but utility-first templates are preferred.

If CSS Modules are necessary:

- Import shared global styles with `@reference` in each module that uses `@apply`.
- Watch build performance because many CSS modules can trigger repeated processing.

## Practical Adoption Decision

- New apps on modern browser baseline: use Tailwind v4 directly.
- Legacy browser products: keep v3.4 until browser policy changes.
- Mixed stack monorepo: standardize one Tailwind entry CSS and explicit references.

<!--
Source references:
- https://tailwindcss.com/docs/compatibility
- https://tailwindcss.com/docs/upgrade-guide
-->

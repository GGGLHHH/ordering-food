# Frontend

React SPA built with Vite+, TanStack Router (file-based routing), and TanStack Query.

## Getting Started

```bash
vp install
vp dev --port 3000
```

## Building For Production

```bash
vp build
```

## Testing

This project uses [Vitest](https://vitest.dev/) (bundled via Vite+) for testing:

```bash
vp test run
```

## Code Quality

```bash
vp check          # format + lint + type check
vp check --fix    # auto-fix
```

## Styling

This project uses [Tailwind CSS](https://tailwindcss.com/) for styling.

### Removing Tailwind CSS

1. Remove the demo pages in `src/routes/demo/`
2. Replace the Tailwind import in `src/styles.css` with your own styles
3. Remove `tailwindcss()` from the plugins array in `vite.config.ts`
4. Uninstall the packages: `vp remove @tailwindcss/vite tailwindcss`

## Shadcn

Add components using the latest version of [Shadcn](https://ui.shadcn.com/):

```bash
vp dlx shadcn@latest add button
```

## Routing

This project uses [TanStack Router](https://tanstack.com/router) with file-based routing. Routes are managed as files in `src/routes`.

To add a new route, create a file in `./src/routes`. TanStack Router will automatically generate the route tree.

Use the `Link` component for SPA navigation:

```tsx
import { Link } from '@tanstack/react-router'
;<Link to="/about">About</Link>
```

More information: [TanStack Router docs](https://tanstack.com/router/latest/docs/framework/react/guide/routing-concepts).

## Demo Files

Files prefixed with `demo` can be safely deleted. They provide a starting point for exploring installed features.

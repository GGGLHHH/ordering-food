---
name: react-ssr-and-hydration
description: Streaming SSR, hydrateRoot, and hydration mismatch prevention in React 19
---

# SSR and Hydration

React server rendering is split into server stream generation and client hydration.

## Node Streaming: `renderToPipeableStream`

```ts
import { renderToPipeableStream } from 'react-dom/server'

app.get('*', (req, res) => {
  const { pipe } = renderToPipeableStream(<App />, {
    bootstrapScripts: ['/main.js'],
    onShellReady() {
      res.setHeader('content-type', 'text/html')
      pipe(res)
    },
  })
})
```

For edge/Web Streams runtimes, use `renderToReadableStream` instead.

## Client Hydration: `hydrateRoot`

```ts
import { hydrateRoot } from 'react-dom/client'

hydrateRoot(document, <App />)
```

Hydration requires the server HTML and first client render output to match.

## Prevent Hydration Mismatches

Common causes:

- Conditional rendering with `typeof window` in render path.
- Reading browser-only APIs during initial render.
- Server/client data divergence.
- Extra whitespace or markup differences around root container.

Fix by moving browser-specific logic into Effects and ensuring deterministic initial render.

## `useId` with Multiple Roots

If using multiple roots, align `identifierPrefix` between server and client root APIs.

```ts
// server + client must share the same prefix
identifierPrefix: 'app-'
```

## Shell vs Full Completion Strategy

- `onShellReady`: start streaming ASAP for faster first paint.
- `onAllReady`: wait for full content (crawler/static style behavior).

<!--
Source references:
- https://react.dev/reference/react-dom/server/renderToPipeableStream
- https://react.dev/reference/react-dom/server/renderToReadableStream
- https://react.dev/reference/react-dom/client/hydrateRoot
-->

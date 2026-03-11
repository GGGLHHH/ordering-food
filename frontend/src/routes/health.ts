import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/health')({
  server: {
    handlers: {
      GET: async () => {
        return new Response('ok', {
          headers: {
            'content-type': 'text/plain; charset=utf-8',
            'cache-control': 'no-store',
          },
        })
      },
    },
  },
})

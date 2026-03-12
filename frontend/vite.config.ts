import tailwindcss from '@tailwindcss/vite'
import { devtools } from '@tanstack/devtools-vite'
import { tanstackStart } from '@tanstack/react-start/plugin/vite'
import viteReact from '@vitejs/plugin-react'
import { nitro } from 'nitro/vite'
import { defineConfig } from 'vite'
import tsconfigPaths from 'vite-tsconfig-paths'

const API_PROXY_TARGET = 'http://127.0.0.1:8080'

const config = defineConfig({
  plugins: [
    devtools(),
    nitro({
      devProxy: {
        '/api/**': {
          changeOrigin: true,
          target: API_PROXY_TARGET,
        },
      },
      rollupConfig: { external: [/^@sentry\//] },
    }),
    tsconfigPaths({ projects: ['./tsconfig.json'] }),
    tailwindcss(),
    tanstackStart(),
    viteReact(),
  ],
})

export default config

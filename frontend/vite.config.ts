import tailwindcss from '@tailwindcss/vite'
import { devtools } from '@tanstack/devtools-vite'
import { tanstackStart } from '@tanstack/react-start/plugin/vite'
import viteReact from '@vitejs/plugin-react'
import { nitro } from 'nitro/vite'
import { defineConfig } from 'vite'
import tsconfigPaths from 'vite-tsconfig-paths'

const API_PROXY_TARGET = 'http://127.0.0.1:8080'
const REACT_QUERY_CLIENT_DIRECTIVE_PATTERN = /@tanstack\/react-query\/build\/modern\/.+\.js$/

function stripReactQueryUseClientDirective() {
  return {
    enforce: 'pre' as const,
    name: 'strip-react-query-use-client-directive',
    transform(code: string, id: string) {
      if (!REACT_QUERY_CLIENT_DIRECTIVE_PATTERN.test(id)) {
        return null
      }

      const strippedCode = code.replace(/^(['"])use client\1;\s*/, '')

      if (strippedCode === code) {
        return null
      }

      return {
        code: strippedCode,
        map: null,
      }
    },
  }
}

const config = defineConfig({
  plugins: [
    stripReactQueryUseClientDirective(),
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

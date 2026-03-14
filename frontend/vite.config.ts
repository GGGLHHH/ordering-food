import tailwindcss from '@tailwindcss/vite'
import { devtools } from '@tanstack/devtools-vite'
import { tanstackRouter } from '@tanstack/router-plugin/vite'
import viteReact from '@vitejs/plugin-react'
import { codeInspectorPlugin } from 'code-inspector-plugin'
import { defineConfig } from 'vite-plus'

const API_PROXY_TARGET = 'http://127.0.0.1:8080'

const config = defineConfig({
  lint: { options: { typeAware: true, typeCheck: true } },
  resolve: {
    tsconfigPaths: true,
  },
  plugins: [
    devtools(),
    tanstackRouter(),
    tailwindcss(),
    viteReact(),
    codeInspectorPlugin({
      bundler: 'vite',
    }),
  ],
  server: {
    proxy: {
      '/api': {
        changeOrigin: true,
        target: API_PROXY_TARGET,
      },
    },
  },
})

export default config

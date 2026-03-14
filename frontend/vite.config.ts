import tailwindcss from '@tailwindcss/vite'
import { devtools } from '@tanstack/devtools-vite'
import { tanstackRouter } from '@tanstack/router-plugin/vite'
import viteReact from '@vitejs/plugin-react'
import { codeInspectorPlugin } from 'code-inspector-plugin'
import { defineConfig } from 'vite-plus'

const API_PROXY_TARGET = 'http://127.0.0.1:8080'

const config = defineConfig({
  lint: {
    plugins: ['react', 'typescript', 'jsx-a11y'],
    options: { typeAware: true, typeCheck: true },
    ignorePatterns: ['src/routeTree.gen.ts', 'src/contracts/generated/**'],
    rules: {
      '@typescript-eslint/no-non-null-assertion': 'off',
      '@typescript-eslint/consistent-type-imports': 'off',
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/no-base-to-string': 'off',
      'jsx-a11y/autocomplete-valid': 'off',
    },
    overrides: [
      {
        files: ['src/routes/__root.tsx'],
        rules: {
          'react/no-danger': 'off',
        },
      },
      {
        files: ['src/components/ui/**'],
        rules: {
          'jsx-a11y/prefer-tag-over-role': 'off',
          'jsx-a11y/interactive-supports-focus': 'off',
          'jsx-a11y/role-has-required-aria-props': 'off',
          'jsx-a11y/no-redundant-roles': 'off',
          'jsx-a11y/click-events-have-key-events': 'off',
          'jsx-a11y/anchor-has-content': 'off',
          'jsx-a11y/autocomplete-valid': 'off',
          'react/no-danger': 'off',
          'react/no-array-index-key': 'off',
          'eslint/no-unused-vars': 'off',
          eqeqeq: 'off',
          'react-hooks/exhaustive-deps': 'off',
        },
      },
    ],
  },
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

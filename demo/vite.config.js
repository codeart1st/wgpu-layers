import { defineConfig, searchForWorkspaceRoot } from 'vite'

export default defineConfig({
  server: {
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp'
    },
    fs: {
      allow: [
        searchForWorkspaceRoot(process.cwd()),
        '../pkg'
      ]
    },
    port: 8080
  }
})
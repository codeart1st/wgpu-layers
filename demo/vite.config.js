import { defineConfig, searchForWorkspaceRoot } from 'vite'

export default defineConfig({
  server: {
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp'
    },
    proxy: {
      '/tegola': {
        target: 'https://tegola-osm-demo.go-spatial.org',
        changeOrigin: true,
        rewrite: path => path.replace(/^\/tegola/, ''),
      },
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
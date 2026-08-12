import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// dev 时 /api 请求代理到后端；生产构建后由后端或独立静态服务
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      '/api': 'http://127.0.0.1:7878',
    },
  },
})

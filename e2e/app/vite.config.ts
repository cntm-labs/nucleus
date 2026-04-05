import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@cntm-labs/react': path.resolve(__dirname, '../../sdks/react/src'),
    },
  },
})

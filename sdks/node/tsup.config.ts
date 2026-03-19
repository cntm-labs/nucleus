import { defineConfig } from 'tsup'

export default defineConfig({
  entry: ['src/index.ts', 'src/express.ts', 'src/fastify.ts', 'src/hono.ts'],
  format: ['esm', 'cjs'],
  dts: true,
  splitting: false,
  clean: true,
  outDir: 'dist',
})

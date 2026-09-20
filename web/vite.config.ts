import path from 'node:path'
import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

// Tauri loads this on http://localhost:1420 in dev, and the built files from ./dist in the installer.
export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: { alias: { '@': path.resolve(import.meta.dirname, './src') } },
  // Tailwind runs through its own Vite plugin. An explicit (empty) PostCSS config also stops Vite
  // from picking up an unrelated postcss.config.js in a parent folder.
  css: { postcss: { plugins: [] } },
  clearScreen: false,
  server: { port: 1420, strictPort: true },
})

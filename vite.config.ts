import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { fileURLToPath, URL } from "node:url";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },

  // Build configuration to handle noVNC top-level await
  build: {
    target: 'esnext',
    rollupOptions: {
      output: {
        format: 'es',
        // Manual chunks for code splitting to reduce bundle size
        manualChunks: {
          // UI framework and components
          'vendor-react': ['react', 'react-dom'],
          'vendor-radix': [
            '@radix-ui/react-dialog',
            '@radix-ui/react-dropdown-menu',
            '@radix-ui/react-context-menu',
            '@radix-ui/react-select',
            '@radix-ui/react-tabs',
            '@radix-ui/react-tooltip',
            '@radix-ui/react-progress',
            '@radix-ui/react-scroll-area',
            '@radix-ui/react-switch',
            '@radix-ui/react-collapsible',
            '@radix-ui/react-checkbox',
            '@radix-ui/react-label',
            '@radix-ui/react-alert-dialog',
          ],
          // Data fetching and state
          'vendor-query': ['@tanstack/react-query'],
          // Tauri APIs
          'vendor-tauri': [
            '@tauri-apps/api',
            '@tauri-apps/plugin-dialog',
          ],
          // Charts library (large)
          'vendor-charts': ['recharts'],
          // Command palette
          'vendor-cmdk': ['cmdk'],
          // Icons and utilities
          'vendor-utils': ['lucide-react', 'clsx', 'tailwind-merge', 'sonner'],
        },
      },
    },
    // Increase chunk size warning limit (appropriate for desktop apps with vendor chunking)
    chunkSizeWarningLimit: 700,
    commonjsOptions: {
      // Exclude noVNC from commonjs processing
      exclude: ['@novnc/novnc/**'],
    },
  },

  // Optimize dependencies - exclude noVNC from pre-bundling
  optimizeDeps: {
    exclude: ['@novnc/novnc'],
    esbuildOptions: {
      target: 'esnext',
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
});

import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import path from "path";

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte(), tailwindcss()],
  // Não limpa o terminal: preserva a saída do compilador Rust do Tauri.
  clearScreen: false,
  resolve: {
    alias: {
      "@": path.resolve(import.meta.dirname, "./src"),
    },
  },
  server: {
    port: 8080,
    strictPort: true,
    watch: {
      // Essencial: não monitorar o src-tauri. O watcher do Vite tentaria observar
      // os binários em src-tauri/target e, no Windows, eles ficam bloqueados
      // durante o build do Rust -> chokidar lança EBUSY e derruba o dev server.
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes("node_modules/@xterm")) {
            return "xterm-vendor";
          }
        },
      },
    },
  },
});

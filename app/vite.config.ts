import { fileURLToPath } from "node:url";
import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vitest/config";

const entry = (path: string) => fileURLToPath(new URL(path, import.meta.url));

export default defineConfig({
  plugins: [vue()],
  // Иначе Vite стирает терминал вместе с ошибками сборки Rust.
  clearScreen: false,
  server: {
    // Tauri ждёт фронт на адресе devUrl из tauri.conf.json.
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  build: {
    rollupOptions: {
      input: { main: entry("windows/main/index.html") },
    },
  },
  test: {
    environment: "jsdom",
  },
});

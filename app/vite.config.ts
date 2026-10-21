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
    coverage: {
      provider: "v8",
      include: ["src/**/*.{ts,vue}"],
      // Точки входа окон только монтируют приложение.
      exclude: ["src/**/*.test.ts", "src/env.d.ts", "src/windows/*/main.ts"],
      reporter: ["text", "lcov"],
      thresholds: { lines: 80, functions: 80, branches: 80, statements: 80 },
    },
  },
});

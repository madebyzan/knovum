import tailwindcss from "@tailwindcss/vite";
import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  build: {
    rollupOptions: {
      input: ["index.html"],
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    ...(host
      ? {
          hmr: {
            protocol: "ws" as const,
            host,
            port: 1421,
          },
        }
      : {}),
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});

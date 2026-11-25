import tailwindcss from "@tailwindcss/vite";
import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite";
import { viteStaticCopy } from "vite-plugin-static-copy";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [
    vue(),
    tailwindcss(),
    // pdf.js fetches its worker, decoders and font data by name at run time.
    viteStaticCopy({
      targets: [
        {
          src: "node_modules/pdfjs-dist/legacy/build/pdf.worker.min.mjs",
          dest: "pdfjs",
          rename: { stripBase: true },
        },
        ...["cmaps", "standard_fonts", "wasm", "iccs"].map((directory) => ({
          src: `node_modules/pdfjs-dist/${directory}/*`,
          dest: `pdfjs/${directory}`,
          rename: { stripBase: true as const },
        })),
      ],
    }),
  ],
  build: {
    rollupOptions: {
      input: ["index.html"],
      output: {
        manualChunks(id) {
          const localeMatch = /\/src\/foundation\/i18n\/locales\/([^/]+)\//.exec(id);
          if (localeMatch) return `locale-${localeMatch[1]}`;
          if (id.includes("/node_modules/pdfjs-dist/")) return "vendor-pdfjs";
          if (id.includes("/node_modules/@tiptap/") || id.includes("/node_modules/prosemirror-")) {
            return "vendor-tiptap";
          }
          if (id.includes("/node_modules/konva/") || id.includes("/node_modules/vue-konva/")) {
            return "vendor-konva";
          }
          if (id.includes("/node_modules/shiki/")) return "vendor-shiki";
          if (id.includes("/node_modules/mediabunny/")) return "vendor-mediabunny";
          return undefined;
        },
      },
    },
  },
  optimizeDeps: {
    include: [
      "@internationalized/date",
      "@tauri-apps/api/app",
      "@tauri-apps/api/core",
      "@tauri-apps/api/dpi",
      "@tauri-apps/api/event",
      "@tauri-apps/api/menu",
      "@tauri-apps/api/webview",
      "@tauri-apps/api/window",
      "@tauri-apps/plugin-dialog",
      "@tauri-apps/plugin-fs",
      "@tauri-apps/plugin-log",
      "@tauri-apps/plugin-opener",
      "@tauri-apps/plugin-os",
      "@tauri-apps/plugin-store",
      "docx-preview",
      "fast-fuzzy",
      "mediabunny",
      "shiki",
      "shiki/engine/javascript",
      "tauri-plugin-clipboard-api",
      "zod",
    ],
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

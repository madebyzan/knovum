import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [vue()],
  test: {
    environment: "happy-dom",
    setupFiles: ["src/foundation/i18n/test-setup.ts"],
    include: ["src/**/*.test.ts"],
    maxWorkers: 4,
    restoreMocks: true,
  },
});

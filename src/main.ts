import { VueQueryPlugin } from "@tanstack/vue-query";
import { error as logError } from "@tauri-apps/plugin-log";
import { platform } from "@tauri-apps/plugin-os";
import { createPinia } from "pinia";
import { createApp } from "vue";

import { bootstrapProduct } from "./app/bootstrap";
import { router } from "./app/router";
import { i18n } from "./foundation/i18n/i18n";
import { queryClient } from "./foundation/query/query-client";

// oxlint-disable-next-line import/no-unassigned-import
import "./styles/app.css";

document.documentElement.dataset.platform = platform();
if (globalThis.matchMedia?.("(prefers-color-scheme: dark)").matches) {
  document.documentElement.dataset.theme = "dark";
}

if (import.meta.env.MODE === "e2e") {
  await import("@wdio/tauri-plugin");
  const { emit } = await import("@tauri-apps/api/event");
  Object.defineProperty(globalThis, "__knovumE2eBridge", {
    value: Object.freeze({ runCommand: (id: string) => emit("knovum-run-command", id) }),
    configurable: false,
    enumerable: false,
    writable: false,
  });
}

const pinia = createPinia();

document.documentElement.dataset.surface = "app";
const { default: App } = await import("./KnovumApp.vue");
const app = createApp(App);
app.config.errorHandler = (err, _instance, info) => {
  const message = err instanceof Error ? err.message : String(err);
  void logError(`Uncaught Vue application error (${info}): ${message}`);
};
app.use(pinia);
app.use(i18n);
app.use(VueQueryPlugin, { queryClient });
app.use(router);
app.mount("#app");
void bootstrapProduct(pinia, router);

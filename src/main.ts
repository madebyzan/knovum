import { VueQueryPlugin } from "@tanstack/vue-query";
import { error as logError } from "@tauri-apps/plugin-log";
import { platform } from "@tauri-apps/plugin-os";
import { createPinia } from "pinia";
import { createApp } from "vue";

import { installContextMenuPolicy } from "./app/context-menu-policy";
import { installLinkPolicy } from "./app/link-policy";
import { formatErrorDiagnostic } from "./foundation/errors/error-format";
import { invokeCommand } from "./foundation/ipc/invoke-command";

// oxlint-disable-next-line import/no-unassigned-import
import "./styles/app.css";

async function start(): Promise<void> {
  installContextMenuPolicy();
  installLinkPolicy();
  document.documentElement.dataset.platform = platform();
  if (globalThis.matchMedia?.("(prefers-color-scheme: dark)").matches) {
    document.documentElement.dataset.theme = "dark";
  }

  if (import.meta.env.MODE === "e2e") {
    await import("@wdio/tauri-plugin");
    const { emit } = await import("@tauri-apps/api/event");
    Object.defineProperty(globalThis, "__knovumE2eBridge", {
      value: Object.freeze({ runCommand: (commandId: string) => emit("knovum-run-command", commandId) }),
      configurable: false,
      enumerable: false,
      writable: false,
    });
  }

  const [{ bootstrap }, { router }, { i18n }, { queryClient }] = await Promise.all([
    import("./app/bootstrap"),
    import("./app/router"),
    import("./foundation/i18n/instance"),
    import("./foundation/query/query-client"),
  ]);
  const pinia = createPinia();

  document.documentElement.dataset.surface = "app";
  const { default: App } = await import("./KnovumApp.vue");
  const app = createApp(App);
  app.config.errorHandler = (err, _instance, info) => {
    void logError(`Uncaught Vue application error (${info}): ${formatErrorDiagnostic(err)}`);
  };
  app.use(pinia);
  app.use(i18n);
  app.use(VueQueryPlugin, { queryClient });
  app.use(router);
  app.mount("#app");
  void bootstrap(pinia, router);
}

void start().catch(async (reason: unknown) => {
  const diagnostic = formatErrorDiagnostic(reason);
  await Promise.allSettled([
    invokeCommand("plugin:desktop|fail_startup", { reason: diagnostic.slice(0, 4096) }),
    logError(`Application entry failed: ${diagnostic}`),
  ]);
});

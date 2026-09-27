import { error as logError } from "@tauri-apps/plugin-log";
import type { Pinia } from "pinia";
import { nextTick } from "vue";
import type { Router } from "vue-router";

import { useBootstrapStore } from "../foundation/bootstrap/bootstrap-store";
import { invokeCommand as invoke } from "../foundation/ipc/invoke-command";
import { initializeExitCoordination } from "../foundation/lifecycle/window-close-coordinator";
import {
  installWorkspaceSessionPersistence,
  restoreWorkspaceSession,
} from "../foundation/navigation/workspace-session";
import { usePreferencesStore } from "../foundation/preferences/preferences-store";
import { restoreResourcePreparationTasks } from "../foundation/runtime/resource-preparation";
import { installTaskbarProgress } from "../foundation/tasks/taskbar-progress";
import { installWindowActivityState } from "../foundation/window/window-activity";
import { installRuntimeDiagnostics } from "./runtime-diagnostics";

const WATCHDOG_TIMEOUT_MS = 25000;

export async function bootstrapProduct(pinia: Pinia, router: Router): Promise<void> {
  const bootstrap = useBootstrapStore(pinia);

  const watchdog = setTimeout(() => {
    bootstrap.fail(new Error("Application startup watchdog timed out after 25s"));
    void invoke("plugin:desktop|reveal_main_window");
  }, WATCHDOG_TIMEOUT_MS);

  try {
    await initializeExitCoordination();
    installRuntimeDiagnostics();
    installTaskbarProgress(pinia);
    void restoreResourcePreparationTasks(pinia).catch((reason) =>
      logError(`Failed to restore resource preparation tasks: ${String(reason)}`),
    );
    await Promise.all([usePreferencesStore(pinia).load(), installWindowActivityState()]);
    await invoke("plugin:spaces|await_space_runtime");
    await restoreWorkspaceSession(router);
    await router.isReady();
    installWorkspaceSessionPersistence(router);
    bootstrap.ready();
    await nextTick();
  } catch (reason) {
    bootstrap.fail(reason);
    await logError(reason instanceof Error ? (reason.stack ?? reason.message) : String(reason));
  } finally {
    clearTimeout(watchdog);
    try {
      await invoke("plugin:desktop|reveal_main_window");
    } catch (reason) {
      await logError(`Failed to complete desktop bootstrap: ${String(reason)}`);
    }
  }
}

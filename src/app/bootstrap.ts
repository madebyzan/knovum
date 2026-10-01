import { error as logError } from "@tauri-apps/plugin-log";
import type { Pinia } from "pinia";
import { nextTick } from "vue";
import type { Router } from "vue-router";

import { useBootstrapStore } from "../foundation/bootstrap/bootstrap-store";
import { formatErrorDiagnostic } from "../foundation/errors/error-format";
import { invokeCommand } from "../foundation/ipc/invoke-command";
import { initializeExitCoordination } from "../foundation/lifecycle/window-close-coordinator";
import {
  installWorkspaceSessionPersistence,
  restoreWorkspaceSession,
} from "../foundation/navigation/workspace-session";
import { usePreferencesStore } from "../foundation/preferences/preferences-store";
import { restoreResourcePreparationTasks } from "../foundation/runtime/resource-preparation";
import { installTaskbarProgress } from "../foundation/tasks/taskbar-progress";
import { useUpdateStore } from "../foundation/updates/update-store";
import { installWindowActivityState } from "../foundation/window/window-activity";
import { installRuntimeDiagnostics } from "./runtime-diagnostics";

const WATCHDOG_TIMEOUT_MS = 25000;

export async function bootstrap(pinia: Pinia, router: Router): Promise<void> {
  const state = useBootstrapStore(pinia);

  const watchdog = setTimeout(() => {
    state.fail(new Error("Application startup watchdog timed out after 25s"));
    void invokeCommand("plugin:desktop|reveal_main_window");
  }, WATCHDOG_TIMEOUT_MS);

  try {
    await initializeExitCoordination();
    installRuntimeDiagnostics();
    installTaskbarProgress(pinia);
    void restoreResourcePreparationTasks(pinia).catch((reason) =>
      logError(`Failed to restore resource preparation tasks: ${formatErrorDiagnostic(reason)}`),
    );
    await Promise.all([usePreferencesStore(pinia).load(), installWindowActivityState()]);
    await invokeCommand("plugin:spaces|await_space_runtime");
    await restoreWorkspaceSession(router);
    await router.isReady();
    installWorkspaceSessionPersistence(router);
    state.ready();
    await nextTick();
    void useUpdateStore(pinia)
      .connect()
      .catch((reason) => logError(`Updater observer failed: ${formatErrorDiagnostic(reason)}`));
  } catch (reason) {
    state.fail(reason);
    await logError(formatErrorDiagnostic(reason));
  } finally {
    clearTimeout(watchdog);
    try {
      await invokeCommand("plugin:desktop|reveal_main_window");
    } catch (reason) {
      await logError(`Failed to reveal main window: ${formatErrorDiagnostic(reason)}`);
    }
  }
}

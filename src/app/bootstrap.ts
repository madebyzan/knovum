import { error as logError } from "@tauri-apps/plugin-log";
import type { Pinia } from "pinia";
import { nextTick } from "vue";
import type { Router } from "vue-router";

import { useBootstrapStore } from "../foundation/bootstrap/bootstrap-store";
import { formatErrorDiagnostic } from "../foundation/errors/error-format";
import { invokeCommand } from "../foundation/ipc/invoke-command";
import { initializeExitCoordination } from "../foundation/lifecycle/window-close-coordinator";
import { usePreferencesStore } from "../foundation/preferences/preferences-store";
import { restoreResourcePreparationTasks } from "../foundation/runtime/resource-preparation";
import { installSessionTaskProjection } from "../foundation/tasks/session-task-projection";
import { installWindowActivityState } from "../foundation/window/window-activity";
import { installRuntimeDiagnostics } from "./runtime-diagnostics";
import { installSettingsPersistence } from "./settings-persistence";
import { installWorkspaceSessionPersistence, restoreWorkspaceSession } from "./workspace-session";

export async function bootstrap(pinia: Pinia, router: Router): Promise<void> {
  const state = useBootstrapStore(pinia);

  try {
    await invokeCommand("plugin:backup|await_maintenance");
    await invokeCommand("plugin:ai|await_ai_runtime");
    installSettingsPersistence(pinia);
    await initializeExitCoordination();
    installRuntimeDiagnostics();
    installSessionTaskProjection(pinia);
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
  } catch (reason) {
    state.fail(reason);
    await logError(formatErrorDiagnostic(reason));
  } finally {
    try {
      await invokeCommand("plugin:desktop|open_main_window");
    } catch (reason) {
      await logError(`Failed to reveal main window: ${formatErrorDiagnostic(reason)}`);
    }
  }
}

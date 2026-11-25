import { listen } from "@tauri-apps/api/event";
import { error as logError } from "@tauri-apps/plugin-log";
import { computed, onMounted, onUnmounted, watchEffect, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import { bindCommandShortcuts } from "../../foundation/commands/command-shortcuts";
import { registerAppCommands, runAppCommandById } from "../../foundation/commands/run-app-command";
import { formatErrorDiagnostic } from "../../foundation/errors/error-format";
import { createAsyncDisposer } from "../../foundation/lifecycle/async-disposer";
import { useShortcutsStore } from "../../foundation/shortcuts/shortcut-store";
import { canCheckForUpdate } from "../../foundation/updates/update-presentation";
import { useUpdateStore } from "../../foundation/updates/update-store";
import type { AppCommand } from "../../shared/contracts/app-command";
import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";
import { createShellCommands } from "./commands";

function resolveOptionalStore<T>(loader: () => T): T | undefined {
  try {
    return loader();
  } catch {
    return undefined;
  }
}

interface ShellCommandControllerOptions {
  workspace: WorkspaceDefinition;
  workspaceActive: Readonly<Ref<boolean>>;
  workspaceCommands: Readonly<Ref<readonly AppCommand[]>>;
  openCommandPalette: () => void;
  openSettings: (pane?: string) => void;
  openShortcuts: () => void;
  toggleSidebar: () => void;
  toggleRightSidebar: () => void;
}

export function useShellCommands(options: ShellCommandControllerOptions) {
  const { t } = useI18n();
  const shortcutsStore = resolveOptionalStore(useShortcutsStore);
  const updateStore = resolveOptionalStore(useUpdateStore);

  function checkForUpdates() {
    options.openSettings("about");
    if (updateStore && canCheckForUpdate(updateStore.state) && !updateStore.requestPending) {
      void updateStore.check().catch((error) => {
        void logError(`Application update check failed: ${formatErrorDiagnostic(error)}`);
      });
    }
  }

  const commands = computed(() =>
    createShellCommands(options.workspace, options.workspaceActive.value, {
      workspaceCommands: options.workspaceCommands.value,
      actions: {
        openCommandPalette: options.openCommandPalette,
        openSettings: () => options.openSettings(),
        checkForUpdates,
        openShortcuts: options.openShortcuts,
        toggleSidebar: options.toggleSidebar,
        toggleRightSidebar: options.toggleRightSidebar,
      },
      labels: {
        editingLabelByCommand: {
          undo: t("common.undo"),
          redo: t("common.redo"),
          cut: t("common.cut"),
          copy: t("common.copy"),
          paste: t("common.paste"),
          "select-all": t("common.selectAll"),
        },
        settings: t("common.settings"),
        checkForUpdates: t("updates.check.action"),
        commandPalette: t("shell.commands.title"),
        shortcuts: t("shell.shortcuts.title"),
        toggleSidebar: t("shell.layout.toggleSidebar"),
        toggleRightSidebar: t("shell.layout.toggleRightSidebar"),
      },
    }),
  );

  const menuCommandListener = createAsyncDisposer();
  watchEffect((cleanup) => {
    cleanup(registerAppCommands(commands.value));
    cleanup(bindCommandShortcuts(commands.value, shortcutsStore?.getEffectiveShortcut));
  });
  onMounted(() => {
    void menuCommandListener
      .attach(listen<string>("knovum-run-command", ({ payload }) => runAppCommandById(payload)))
      .catch(
        (error) => void logError(`Failed to register menu command listener: ${formatErrorDiagnostic(error)}`),
      );
  });
  onUnmounted(menuCommandListener.dispose);

  return commands;
}

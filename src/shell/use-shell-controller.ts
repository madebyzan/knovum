import { listen } from "@tauri-apps/api/event";
import { error as logError } from "@tauri-apps/plugin-log";
import { useTimeoutFn } from "@vueuse/core";
import { computed, nextTick, onMounted, onUnmounted, provide, shallowRef, watchEffect } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";

import { bindCommandShortcuts } from "../foundation/commands/command-shortcuts";
import { runAppCommand } from "../foundation/commands/run-app-command";
import { createAsyncDisposer } from "../foundation/lifecycle/async-disposer";
import type { WorkspaceDefinition } from "../shared/contracts/workspace-definition";
import { workspaceRightSidebarKey } from "../shared/contracts/workspace-right-sidebar";
import { workspaceSidebarKey } from "../shared/contracts/workspace-sidebar";
import { createShellCommands } from "./commands";
import { workspaceSidebarGeometry } from "./sidebar-geometry";

export function useShellController(workspace: WorkspaceDefinition) {
  const route = useRoute();
  const router = useRouter();
  const { t } = useI18n();
  const commandOpen = shallowRef(false);
  const sidebarPinned = shallowRef(true);
  const sidebarWidth = shallowRef<number>(workspaceSidebarGeometry.defaultSize);
  const sidebarPreview = shallowRef(false);
  const sidebarPreviewArmed = shallowRef(true);
  const rightSidebarOpen = shallowRef(false);
  const rightSidebarWidth = shallowRef(workspace.rightSidebarLayout.defaultSize);
  const workspaceActive = computed(() => route.name === "spaces");
  const sidebarVisible = computed(
    () => workspaceActive.value && (sidebarPinned.value || sidebarPreview.value),
  );
  const { start: startSidebarPreviewClose, stop: cancelSidebarPreviewClose } = useTimeoutFn(
    () => (sidebarPreview.value = false),
    160,
    { immediate: false },
  );

  function toggleSidebarPanel() {
    setSidebarPinned(!sidebarPinned.value);
  }
  function toggleSidebarFromControl() {
    if (sidebarPinned.value) sidebarPreviewArmed.value = false;
    toggleSidebarPanel();
  }
  function setSidebarPinned(value: boolean) {
    sidebarPinned.value = value;
    sidebarPreview.value = false;
    if (value) sidebarPreviewArmed.value = true;
    cancelSidebarPreviewClose();
  }
  function setSidebarWidth(value: number) {
    sidebarWidth.value = Math.round(value);
  }
  function openSidebarPreview() {
    if (sidebarPinned.value || !sidebarPreviewArmed.value) return;
    cancelSidebarPreviewClose();
    sidebarPreview.value = true;
  }
  function armSidebarPreview() {
    sidebarPreviewArmed.value = true;
    scheduleSidebarPreviewClose();
  }
  function scheduleSidebarPreviewClose() {
    if (sidebarPinned.value) return;
    cancelSidebarPreviewClose();
    startSidebarPreviewClose();
  }
  function closeSidebarPreview() {
    cancelSidebarPreviewClose();
    sidebarPreview.value = false;
  }
  function toggleRightSidebar() {
    if (!workspaceActive.value || !workspace.rightSidebarAvailable.value) return;
    if (rightSidebarOpen.value) {
      rightSidebarOpen.value = false;
    } else {
      workspace.openDefaultRightSidebar();
      rightSidebarOpen.value = true;
    }
  }
  function openRightSidebar() {
    if (!workspaceActive.value || !workspace.rightSidebarAvailable.value) return;
    rightSidebarOpen.value = true;
  }
  function setRightSidebarWidth(value: number) {
    rightSidebarWidth.value = Math.round(value);
  }
  function closeRightSidebar() {
    rightSidebarOpen.value = false;
    void nextTick(() => document.getElementById("workspace-right-sidebar-toggle")?.focus());
  }
  provide(workspaceRightSidebarKey, { open: openRightSidebar, close: closeRightSidebar });
  watchEffect(() => {
    if (!workspace.rightSidebarAvailable.value) rightSidebarOpen.value = false;
  });
  provide(workspaceSidebarKey, { reveal: () => setSidebarPinned(true) });
  const shortcutsOpen = shallowRef(false);
  const commands = computed(() =>
    createShellCommands(workspace, workspaceActive.value, {
      actions: {
        openCommandPalette: () => (commandOpen.value = true),
        openSettings: () => void router.push("/settings"),
        openShortcuts: () => (shortcutsOpen.value = true),
        toggleSidebar: toggleSidebarPanel,
        toggleRightSidebar,
      },
      labels: {
        settings: t("settings.general.title"),
        commandPalette: t("shell.commandPalette"),
        shortcuts: t("shell.keyboardShortcuts"),
        toggleSidebar: t("shell.toggleSidebar"),
        toggleRightSidebar: t("shell.toggleRightSidebar"),
      },
    }),
  );
  function runCommand(id: string) {
    const command = commands.value.find((candidate) => candidate.id === id);
    if (command) runAppCommand(command);
  }
  const menuCommandListener = createAsyncDisposer();
  watchEffect((cleanup) => cleanup(bindCommandShortcuts(commands.value)));
  onMounted(() => {
    void menuCommandListener
      .attach(listen<string>("knovum-run-command", ({ payload }) => runCommand(payload)))
      .catch((error) => void logError(`Failed to register menu command listener: ${String(error)}`));
  });
  onUnmounted(menuCommandListener.dispose);
  return {
    route,
    commandOpen,
    shortcutsOpen,
    sidebarVisible,
    sidebarPinned,
    sidebarWidth,
    sidebarPreview,
    rightSidebarOpen,
    rightSidebarWidth,
    workspaceActive,
    commands,
    toggleSidebarPanel,
    toggleSidebarFromControl,
    setSidebarPinned,
    setSidebarWidth,
    setRightSidebarWidth,
    openSidebarPreview,
    armSidebarPreview,
    scheduleSidebarPreviewClose,
    closeSidebarPreview,
    toggleRightSidebar,
    closeRightSidebar,
  };
}

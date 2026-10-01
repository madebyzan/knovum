import { listen } from "@tauri-apps/api/event";
import { error as logError } from "@tauri-apps/plugin-log";
import { useEventListener, useTimeoutFn } from "@vueuse/core";
import { computed, onMounted, onUnmounted, provide, shallowRef, watchEffect } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";

import { bindCommandShortcuts } from "../foundation/commands/command-shortcuts";
import { runAppCommand } from "../foundation/commands/run-app-command";
import { errorDiagnostic } from "../foundation/errors/error-format";
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
  const panelMotion = shallowRef(false);
  const { start: endPanelMotion, stop: cancelPanelMotionTimer } = useTimeoutFn(
    () => (panelMotion.value = false),
    220,
    { immediate: false },
  );
  function beginPanelMotion() {
    panelMotion.value = true;
    endPanelMotion();
  }
  function cancelPanelMotion() {
    cancelPanelMotionTimer();
    panelMotion.value = false;
  }
  useEventListener(globalThis, "resize", cancelPanelMotion);
  const workspaceActive = computed(() => route.name === "spaces");
  const { start: startSidebarPreviewClose, stop: cancelSidebarPreviewClose } = useTimeoutFn(
    () => (sidebarPreview.value = false),
    160,
    { immediate: false },
  );

  function toggleSidebarPanel() {
    beginPanelMotion();
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
    beginPanelMotion();
    if (rightSidebarOpen.value) {
      rightSidebarOpen.value = false;
    } else {
      workspace.openDefaultRightSidebar();
      rightSidebarOpen.value = true;
    }
  }
  function openRightSidebar() {
    if (!workspaceActive.value || !workspace.rightSidebarAvailable.value) return;
    if (!rightSidebarOpen.value) beginPanelMotion();
    rightSidebarOpen.value = true;
  }
  function setRightSidebarWidth(value: number) {
    rightSidebarWidth.value = Math.round(value);
  }
  function closeRightSidebar() {
    rightSidebarOpen.value = false;
    document.getElementById("workspace-right-sidebar-toggle")?.focus();
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
        settings: t("common.settings"),
        commandPalette: t("shell.commands.title"),
        shortcuts: t("shell.shortcuts.title"),
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
      .catch((error) => void logError(`Failed to register menu command listener: ${errorDiagnostic(error)}`));
  });
  onUnmounted(menuCommandListener.dispose);
  return {
    route,
    commandOpen,
    shortcutsOpen,
    sidebarPinned,
    sidebarWidth,
    sidebarPreview,
    rightSidebarOpen,
    rightSidebarWidth,
    panelMotion,
    cancelPanelMotion,
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

import { listen } from "@tauri-apps/api/event";
import { error as logError } from "@tauri-apps/plugin-log";
import { useEventListener, useTimeoutFn } from "@vueuse/core";
import { computed, onMounted, onUnmounted, provide, shallowRef, watchEffect } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";

import { bindCommandShortcuts } from "../foundation/commands/command-shortcuts";
import { registerAppCommands, runAppCommandById } from "../foundation/commands/run-app-command";
import { formatErrorDiagnostic } from "../foundation/errors/error-format";
import { createAsyncDisposer } from "../foundation/lifecycle/async-disposer";
import type { WorkspaceDefinition } from "../shared/contracts/workspace-definition";
import { workspaceRightSidebarKey } from "../shared/contracts/workspace-right-sidebar";
import { workspaceSidebarKey } from "../shared/contracts/workspace-sidebar";
import { createShellCommands } from "./commands";
import { useWorkbenchLayoutPreferences } from "./use-workbench-layout-preferences";

export function useShellController(workspace: WorkspaceDefinition) {
  const route = useRoute();
  const router = useRouter();
  const { t } = useI18n();
  const commandOpen = shallowRef(false);
  const {
    sidebarPinned,
    sidebarWidth,
    rightSidebarOpen: rightSidebarRequested,
    rightSidebarWidth,
    persist,
  } = useWorkbenchLayoutPreferences();
  const rightSidebarOpen = computed(
    () => rightSidebarRequested.value && workspace.rightSidebarAvailable.value,
  );
  const sidebarPreview = shallowRef(false);
  const sidebarPreviewArmed = shallowRef(true);
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
    persist();
  }
  function setSidebarWidth(value: number) {
    sidebarWidth.value = Math.round(value);
    persist();
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
      rightSidebarRequested.value = false;
    } else {
      workspace.openDefaultRightSidebar();
      rightSidebarRequested.value = true;
    }
    persist();
  }
  function openRightSidebar() {
    if (!workspaceActive.value || !workspace.rightSidebarAvailable.value) return;
    if (!rightSidebarOpen.value) beginPanelMotion();
    rightSidebarRequested.value = true;
    persist();
  }
  function setRightSidebarWidth(value: number) {
    rightSidebarWidth.value = Math.round(value);
    persist();
  }
  function closeRightSidebar() {
    rightSidebarRequested.value = false;
    persist();
    document.getElementById("workspace-right-sidebar-toggle")?.focus();
  }
  provide(workspaceRightSidebarKey, { open: openRightSidebar, close: closeRightSidebar });
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
        copy: t("common.copy"),
        cut: t("common.cut"),
        paste: t("common.paste"),
        settings: t("common.settings"),
        commandPalette: t("shell.commands.title"),
        shortcuts: t("shell.shortcuts.title"),
        toggleSidebar: t("shell.toggleSidebar"),
        toggleRightSidebar: t("shell.toggleRightSidebar"),
      },
    }),
  );
  const menuCommandListener = createAsyncDisposer();
  watchEffect((cleanup) => {
    cleanup(registerAppCommands(commands.value));
    cleanup(bindCommandShortcuts(commands.value));
  });
  onMounted(() => {
    void menuCommandListener
      .attach(listen<string>("knovum-run-command", ({ payload }) => runAppCommandById(payload)))
      .catch(
        (error) => void logError(`Failed to register menu command listener: ${formatErrorDiagnostic(error)}`),
      );
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

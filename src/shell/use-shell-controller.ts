import { listen } from "@tauri-apps/api/event";
import { error as logError } from "@tauri-apps/plugin-log";
import { useTimeoutFn } from "@vueuse/core";
import { computed, onMounted, onUnmounted, provide, shallowRef, watchEffect } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";

import { bindCommandShortcuts } from "../foundation/commands/command-shortcuts";
import { runAppCommand } from "../foundation/commands/run-app-command";
import { createAsyncDisposer } from "../foundation/lifecycle/async-disposer";
import type { WorkspaceDefinition } from "../shared/contracts/workspace-definition";
import {
  workspaceInspectorKey,
  type WorkspaceInspectorPresentation,
} from "../shared/contracts/workspace-inspector";
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
  const inspectorOpen = shallowRef(false);
  const inspectorWidth = shallowRef(workspace.inspectorLayout.minSize);
  const inspectorPresentation = shallowRef<WorkspaceInspectorPresentation>("floating");
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
    sidebarWidth.value = value;
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
  function toggleInspectorPanel() {
    if (!workspaceActive.value) return;
    inspectorOpen.value = !inspectorOpen.value;
  }
  function openInspectorPanel(
    preferredSize?: number,
    presentation: WorkspaceInspectorPresentation = "floating",
  ) {
    if (!workspaceActive.value) return;
    if (preferredSize !== undefined) inspectorWidth.value = preferredSize;
    inspectorPresentation.value = presentation;
    inspectorOpen.value = true;
  }
  function setInspectorWidth(value: number) {
    inspectorWidth.value = value;
  }
  function closeInspectorPanel() {
    inspectorOpen.value = false;
  }
  provide(workspaceInspectorKey, { open: openInspectorPanel, close: closeInspectorPanel });
  provide(workspaceSidebarKey, { reveal: () => setSidebarPinned(true) });
  const shortcutsOpen = shallowRef(false);
  const commands = computed(() =>
    createShellCommands(workspace, workspaceActive.value, {
      actions: {
        openCommandPalette: () => (commandOpen.value = true),
        openSettings: () => void router.push("/settings"),
        openShortcuts: () => (shortcutsOpen.value = true),
        toggleSidebar: toggleSidebarPanel,
        toggleInspector: toggleInspectorPanel,
      },
      labels: {
        settings: t("settings.general.title"),
        commandPalette: t("shell.commandPalette"),
        shortcuts: t("shell.keyboardShortcuts"),
        toggleSidebar: t("shell.toggleSidebar"),
        toggleInspector: t("shell.toggleInspector"),
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
    inspectorOpen,
    inspectorWidth,
    inspectorPresentation,
    workspaceActive,
    commands,
    toggleSidebarPanel,
    toggleSidebarFromControl,
    setSidebarPinned,
    setSidebarWidth,
    setInspectorWidth,
    openSidebarPreview,
    armSidebarPreview,
    scheduleSidebarPreviewClose,
    closeSidebarPreview,
    toggleInspectorPanel,
    closeInspectorPanel,
  };
}

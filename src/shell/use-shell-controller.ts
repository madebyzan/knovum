import { useTimeoutFn } from "@vueuse/core";
import { computed, provide, shallowRef, type Ref } from "vue";

import type { WorkspaceDefinition } from "../shared/contracts/workspace-definition";
import { workspaceRightSidebarKey } from "../shared/contracts/workspace-right-sidebar";
import { workspaceSidebarKey } from "../shared/contracts/workspace-sidebar";
import { useShellCommands } from "./commands/use-shell-commands";
import { useWorkbenchLayoutPreferences } from "./layout/use-workbench-layout-preferences";

export function useShellController(
  workspace: WorkspaceDefinition,
  workspaceActive: Readonly<Ref<boolean>>,
  openSettings: (pane?: string) => void,
) {
  const commandOpen = shallowRef(false);
  const shortcutsOpen = shallowRef(false);
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
  const rightSidebarContentMounted = computed(() => rightSidebarOpen.value);
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
    persist();
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
      rightSidebarRequested.value = false;
    } else {
      workspace.openDefaultRightSidebar();
      rightSidebarRequested.value = true;
    }
    persist();
  }
  function openRightSidebar() {
    if (!workspaceActive.value || !workspace.rightSidebarAvailable.value) return;
    rightSidebarRequested.value = true;
    persist();
  }
  function setRightSidebarWidth(value: number) {
    rightSidebarWidth.value = Math.round(value);
  }
  function closeRightSidebar() {
    rightSidebarRequested.value = false;
    persist();
    document.getElementById("workspace-right-sidebar-toggle")?.focus();
  }
  const rightSidebar = { open: openRightSidebar, close: closeRightSidebar };
  const sidebar = {
    pinned: computed(() => sidebarPinned.value),
    reveal: () => setSidebarPinned(true),
    collapse: () => setSidebarPinned(false),
  };
  provide(workspaceRightSidebarKey, rightSidebar);
  provide(workspaceSidebarKey, sidebar);
  const workspaceCommands = workspace.useCommands({ sidebar, rightSidebar });
  const commands = useShellCommands({
    workspace,
    workspaceActive,
    workspaceCommands,
    openCommandPalette: () => (commandOpen.value = true),
    openSettings,
    openShortcuts: () => (shortcutsOpen.value = true),
    toggleSidebar: toggleSidebarPanel,
    toggleRightSidebar,
  });
  return {
    commandOpen,
    shortcutsOpen,
    sidebarPinned,
    sidebarWidth,
    sidebarPreview,
    rightSidebarOpen,
    rightSidebarContentMounted,
    rightSidebarWidth,
    commands,
    toggleSidebarFromControl,
    setSidebarPinned,
    setSidebarWidth,
    setRightSidebarWidth,
    /** A sash writes the width on every frame of the gesture; only the release reaches preferences. */
    persistLayout: persist,
    openSidebarPreview,
    armSidebarPreview,
    scheduleSidebarPreviewClose,
    closeSidebarPreview,
    toggleRightSidebar,
    closeRightSidebar,
  };
}

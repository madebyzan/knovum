<script setup lang="ts">
import { RouterView } from "vue-router";

import type { WorkspaceDefinition } from "../shared/contracts/workspace-definition";
import CommandPalette from "./components/CommandPalette.vue";
import KeyboardShortcutsModal from "./components/KeyboardShortcutsModal.vue";
import WorkspaceLayout from "./components/WorkspaceLayout.vue";
import WorkspaceToolbar from "./components/WorkspaceToolbar.vue";
import FeatureErrorBoundary from "./FeatureErrorBoundary.vue";
import { useShellController } from "./use-shell-controller";

const props = defineProps<{ workspace: WorkspaceDefinition }>();
const {
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
  toggleSidebarFromControl,
  toggleRightSidebar,
  setSidebarPinned,
  setSidebarWidth,
  setRightSidebarWidth,
  openSidebarPreview,
  armSidebarPreview,
  scheduleSidebarPreviewClose,
  closeSidebarPreview,
} = useShellController(props.workspace);
</script>

<template>
  <div
    v-if="!workspaceActive"
    class="absolute inset-0 z-10 h-full min-h-0 w-full min-w-0 overflow-hidden bg-surface-canvas"
  >
    <RouterView v-slot="{ Component }">
      <FeatureErrorBoundary :reset-key="route.fullPath">
        <component :is="Component" key="settings" />
      </FeatureErrorBoundary>
    </RouterView>
  </div>
  <div
    data-testid="spaces-workspace"
    class="spaces-workspace absolute inset-0 flex h-full min-h-0 w-full min-w-0 flex-col overflow-hidden bg-surface-chrome transition-colors duration-220 ease-standard select-none"
    :class="{ invisible: !workspaceActive }"
    :aria-hidden="!workspaceActive"
    :inert="!workspaceActive"
  >
    <WorkspaceToolbar
      :workspace="workspace"
      :sidebar-pinned="sidebarPinned"
      :sidebar-width="sidebarWidth"
      :right-sidebar-open="rightSidebarOpen"
      :right-sidebar-width="rightSidebarWidth"
      @toggle-right-sidebar="toggleRightSidebar"
      @toggle-sidebar="toggleSidebarFromControl"
      @preview-sidebar="openSidebarPreview"
      @cancel-sidebar-preview="armSidebarPreview"
    />
    <WorkspaceLayout
      :workspace="workspace"
      :sidebar-visible="sidebarVisible"
      :sidebar-pinned="sidebarPinned"
      :sidebar-width="sidebarWidth"
      :sidebar-preview="sidebarPreview"
      :right-sidebar-open="rightSidebarOpen"
      :right-sidebar-width="rightSidebarWidth"
      @set-sidebar-pinned="setSidebarPinned"
      @set-sidebar-width="setSidebarWidth"
      @set-right-sidebar-width="setRightSidebarWidth"
      @open-sidebar-preview="openSidebarPreview"
      @schedule-sidebar-close="scheduleSidebarPreviewClose"
      @close-sidebar-preview="closeSidebarPreview"
    />
  </div>
  <CommandPalette v-model:open="commandOpen" :commands="commands" />
  <KeyboardShortcutsModal v-model:open="shortcutsOpen" :commands="commands" />
</template>

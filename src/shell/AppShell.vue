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
  inspectorOpen,
  inspectorWidth,
  inspectorPresentation,
  workspaceActive,
  commands,
  toggleSidebarFromControl,
  setSidebarPinned,
  setSidebarWidth,
  setInspectorWidth,
  openSidebarPreview,
  armSidebarPreview,
  scheduleSidebarPreviewClose,
  closeSidebarPreview,
} = useShellController(props.workspace);
</script>

<template>
  <div
    v-if="!workspaceActive"
    class="absolute inset-0 z-10 h-full min-h-0 w-full min-w-0 overflow-hidden bg-canvas"
  >
    <RouterView v-slot="{ Component }">
      <FeatureErrorBoundary :reset-key="route.fullPath">
        <component :is="Component" key="settings" />
      </FeatureErrorBoundary>
    </RouterView>
  </div>
  <div
    data-testid="spaces-workspace"
    class="absolute inset-0 flex h-full min-h-0 w-full min-w-0 flex-col overflow-hidden bg-chrome transition-colors duration-200 ease-standard select-none"
    :class="{ invisible: !workspaceActive }"
    :aria-hidden="!workspaceActive"
    :inert="!workspaceActive"
  >
    <WorkspaceToolbar
      :workspace="workspace"
      :sidebar-pinned="sidebarPinned"
      :sidebar-width="sidebarWidth"
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
      :inspector-open="inspectorOpen"
      :inspector-width="inspectorWidth"
      :inspector-presentation="inspectorPresentation"
      @set-sidebar-pinned="setSidebarPinned"
      @set-sidebar-width="setSidebarWidth"
      @set-inspector-width="setInspectorWidth"
      @open-sidebar-preview="openSidebarPreview"
      @schedule-sidebar-close="scheduleSidebarPreviewClose"
      @close-sidebar-preview="closeSidebarPreview"
    />
  </div>
  <CommandPalette v-model:open="commandOpen" :commands="commands" />
  <KeyboardShortcutsModal v-model:open="shortcutsOpen" :commands="commands" />
</template>

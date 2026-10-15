<script setup lang="ts">
import { computed, provide, toRef } from "vue";
import { RouterView, useRoute } from "vue-router";

import { overlayScopeVisibleKey } from "../shared/contracts/overlay-scope-visibility";
import type { WorkspaceDefinition } from "../shared/contracts/workspace-definition";
import CommandPalette from "./commands/CommandPalette.vue";
import KeyboardShortcutsModal from "./commands/KeyboardShortcutsModal.vue";
import FeatureErrorBoundary from "./FeatureErrorBoundary.vue";
import WorkspaceLayout from "./layout/WorkspaceLayout.vue";
import WorkspaceToolbar from "./layout/WorkspaceToolbar.vue";
import { useShellController } from "./use-shell-controller";

const props = defineProps<{ workspace: WorkspaceDefinition; workspaceActive: boolean }>();
const emit = defineEmits<{ openSettings: [] }>();
const route = useRoute();
const workspaceActive = toRef(props, "workspaceActive");
const {
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
  toggleRightSidebar,
  setSidebarWidth,
  setRightSidebarWidth,
  persistLayout,
  openSidebarPreview,
  armSidebarPreview,
  scheduleSidebarPreviewClose,
  closeSidebarPreview,
} = useShellController(props.workspace, workspaceActive, () => emit("openSettings"));
provide(overlayScopeVisibleKey, workspaceActive);
/** The toolbar's trailing column tracks the pane it aligns with, and collapses to nothing when it closes. */
const rightSidebarLayoutWidth = computed(() => (rightSidebarOpen.value ? rightSidebarWidth.value : 0));
</script>

<template>
  <div
    v-if="!workspaceActive"
    class="absolute inset-0 z-10 h-full min-h-0 w-full min-w-0 overflow-hidden bg-surface-canvas"
  >
    <RouterView v-slot="{ Component }">
      <FeatureErrorBoundary :reset-key="route.fullPath">
        <component :is="Component" />
      </FeatureErrorBoundary>
    </RouterView>
  </div>
  <!-- Preserve workspace geometry while an application destination covers it. -->
  <div
    data-testid="workspace"
    class="workspace absolute inset-0 flex h-full min-h-0 w-full min-w-0 flex-col overflow-hidden bg-surface-chrome transition-colors duration-220 ease-standard select-none"
    :class="{ 'opacity-0': !workspaceActive }"
    :aria-hidden="!workspaceActive"
    :inert="!workspaceActive"
  >
    <WorkspaceToolbar
      :workspace="workspace"
      :sidebar-pinned="sidebarPinned"
      :sidebar-width="sidebarWidth"
      :right-sidebar-open="rightSidebarOpen"
      :right-sidebar-layout-width="rightSidebarLayoutWidth"
      @toggle-right-sidebar="toggleRightSidebar"
      @toggle-sidebar="toggleSidebarFromControl"
      @preview-sidebar="openSidebarPreview"
      @cancel-sidebar-preview="armSidebarPreview"
    />
    <WorkspaceLayout
      :workspace="workspace"
      :sidebar-pinned="sidebarPinned"
      :sidebar-width="sidebarWidth"
      :sidebar-preview="sidebarPreview"
      :right-sidebar-open="rightSidebarOpen"
      :right-sidebar-content-mounted="rightSidebarContentMounted"
      :right-sidebar-width="rightSidebarWidth"
      @set-sidebar-width="setSidebarWidth"
      @set-right-sidebar-width="setRightSidebarWidth"
      @commit-layout="persistLayout"
      @open-sidebar-preview="openSidebarPreview"
      @schedule-sidebar-close="scheduleSidebarPreviewClose"
      @close-sidebar-preview="closeSidebarPreview"
    />
  </div>
  <CommandPalette v-model:open="commandOpen" :commands="commands" />
  <KeyboardShortcutsModal v-model:open="shortcutsOpen" :commands="commands" />
</template>

<script setup lang="ts">
import { computed } from "vue";

import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";
import UiIconButton from "../../shared/ui/UiIconButton.vue";
import UiPanelToggleIcon from "../../shared/ui/UiPanelToggleIcon.vue";
import TaskCenter from "../tasks/TaskCenter.vue";
import type { WorkspaceEdges } from "./use-workspace-edges";
import WorkspaceNavigationControls from "./WorkspaceNavigationControls.vue";

const props = defineProps<{
  workspace: WorkspaceDefinition;
  sidebarPinned: boolean;
  rightSidebarOpen: boolean;
  edges: WorkspaceEdges;
}>();
/**
 * The leading column reserves at least the navigation cluster and otherwise ends exactly where the canvas
 * begins - the same expression as the sidebar slot, on the same frame of the panel clock - so the first tab
 * rides the canvas edge once it passes the cluster and waits for it before. The trailing column tracks the
 * pane it aligns with and never gets narrower than the utilities it holds.
 */
const toolbarColumns = computed(
  () =>
    `minmax(auto, max(var(--knovum-workspace-gutter), ${props.edges.leading}px)) minmax(0, 1fr) ` +
    `${Math.max(props.edges.trailing, 96)}px`,
);
const emit = defineEmits<{
  "toggle-sidebar": [];
  "toggle-right-sidebar": [];
  "preview-sidebar": [];
  "cancel-sidebar-preview": [];
}>();
</script>

<template>
  <header
    class="@container relative h-(--knovum-titlebar-height) shrink-0 material-toolbar [container-name:workspace-toolbar]"
  >
    <div data-tauri-drag-region class="absolute inset-0" />
    <div
      class="workspace-toolbar-content relative grid h-full"
      :style="{ gridTemplateColumns: toolbarColumns }"
    >
      <div class="flex min-w-max items-center gap-1 pr-2 pl-(--knovum-titlebar-navigation-padding-start)">
        <WorkspaceNavigationControls
          :sidebar-pinned="sidebarPinned"
          @toggle-sidebar="emit('toggle-sidebar')"
          @preview-sidebar="emit('preview-sidebar')"
          @cancel-sidebar-preview="emit('cancel-sidebar-preview')"
        />
        <component :is="workspace.toolbarLeading" class="ml-auto shrink-0" />
      </div>
      <div class="flex min-w-0 items-center pr-2">
        <component :is="workspace.toolbar" class="min-w-0 flex-1" />
      </div>
      <div
        class="z-10 col-start-3 row-start-1 flex min-w-max items-center gap-1 self-center justify-self-end pr-3"
      >
        <TaskCenter :current-scope="workspace.currentScope?.value" />
        <UiIconButton
          v-if="workspace.rightSidebarAvailable.value"
          id="workspace-right-sidebar-toggle"
          :label="$t('shell.layout.toggleRightSidebar')"
          aria-controls="spaces-right-sidebar"
          :aria-expanded="rightSidebarOpen"
          shortcut="$mod+Shift+B"
          size="md"
          @click="emit('toggle-right-sidebar')"
        >
          <UiPanelToggleIcon class="size-4.5" side="right" :open="rightSidebarOpen" />
        </UiIconButton>
      </div>
    </div>
  </header>
</template>

<style scoped>
.workspace-toolbar-content {
  pointer-events: none;
}
.workspace-toolbar-content :deep(button),
.workspace-toolbar-content :deep(input),
.workspace-toolbar-content :deep(select),
.workspace-toolbar-content :deep(textarea),
.workspace-toolbar-content :deep(a) {
  pointer-events: auto;
}
</style>

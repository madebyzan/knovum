<script setup lang="ts">
import { PanelRightClose, PanelRightOpen } from "@lucide/vue";
import { computed } from "vue";

import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";
import UiIconButton from "../../shared/ui/UiIconButton.vue";
import TaskCenter from "../tasks/TaskCenter.vue";
import WorkspaceNavigationControls from "./WorkspaceNavigationControls.vue";

const props = defineProps<{
  workspace: WorkspaceDefinition;
  sidebarPinned: boolean;
  sidebarWidth: number;
  rightSidebarOpen: boolean;
  rightSidebarLayoutWidth: number;
}>();
const trailingUtilityWidth = computed(() => Math.max(props.rightSidebarLayoutWidth, 96));
/**
 * The leading column reserves at least the navigation cluster and otherwise ends exactly where the canvas
 * begins, so the first tab lines up with the work area's leading edge. Its limit is the same edge the
 * sidebar slot moves - the sidebar width when pinned, the work area gutter otherwise - interpolated on the
 * same clock, so the tabs ride the canvas edge once it passes the cluster and wait for it before.
 */
const leadingColumnLimit = computed(() =>
  props.sidebarPinned ? `${props.sidebarWidth}px` : "var(--knovum-workspace-gutter)",
);
const toolbarColumns = computed(
  () => `minmax(auto, ${leadingColumnLimit.value}) minmax(0, 1fr) ${trailingUtilityWidth.value}px`,
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
          :active="rightSidebarOpen"
          size="md"
          @click="emit('toggle-right-sidebar')"
        >
          <component
            :is="rightSidebarOpen ? PanelRightClose : PanelRightOpen"
            class="size-4.5"
            :stroke-width="1.55"
          />
        </UiIconButton>
      </div>
    </div>
  </header>
</template>

<style scoped>
.workspace-toolbar-content {
  pointer-events: none;
  transition: grid-template-columns var(--knovum-panel-motion-duration) var(--knovum-panel-motion-easing);
}
html[data-sash-dragging] .workspace-toolbar-content {
  transition: none;
}
.workspace-toolbar-content :deep(button),
.workspace-toolbar-content :deep(input),
.workspace-toolbar-content :deep(select),
.workspace-toolbar-content :deep(textarea),
.workspace-toolbar-content :deep(a) {
  pointer-events: auto;
}
</style>

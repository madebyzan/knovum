<script setup lang="ts">
import { PanelRightClose, PanelRightOpen } from "@lucide/vue";
import { computed } from "vue";

import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";
import UiIconButton from "../../shared/ui/UiIconButton.vue";
import WorkspaceNavigationControls from "./WorkspaceNavigationControls.vue";

const props = defineProps<{
  workspace: WorkspaceDefinition;
  sidebarPinned: boolean;
  sidebarWidth: number;
  rightSidebarOpen: boolean;
  rightSidebarLayoutWidth: number;
}>();
const toolbarColumns = computed(
  () =>
    `${props.sidebarWidth}px minmax(0, 1fr) ${props.rightSidebarLayoutWidth}px`,
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
      <div class="flex min-w-0 items-center gap-1 pr-2 pl-(--knovum-titlebar-navigation-padding-start)">
        <WorkspaceNavigationControls
          :sidebar-pinned="sidebarPinned"
          @toggle-sidebar="emit('toggle-sidebar')"
          @preview-sidebar="emit('preview-sidebar')"
          @cancel-sidebar-preview="emit('cancel-sidebar-preview')"
        />
        <component :is="workspace.toolbarLeading" class="ml-auto shrink-0" />
      </div>
      <div class="flex min-w-0 items-center pl-2" :class="rightSidebarOpen ? 'pr-3' : 'pr-14'">
        <component :is="workspace.toolbar" class="min-w-0 flex-1" />
      </div>
    </div>
    <UiIconButton
      id="workspace-right-sidebar-toggle"
      class="absolute top-1/2 right-3 -translate-y-1/2"
      :label="$t('shell.toggleRightSidebar')"
      aria-controls="spaces-right-sidebar"
      :aria-expanded="rightSidebarOpen"
      shortcut="$mod+Shift+B"
      :active="rightSidebarOpen"
      :disabled="!workspace.rightSidebarAvailable.value"
      size="md"
      @click="emit('toggle-right-sidebar')"
    >
      <component
        :is="rightSidebarOpen ? PanelRightClose : PanelRightOpen"
        class="size-4.5"
        :stroke-width="1.55"
      />
    </UiIconButton>
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

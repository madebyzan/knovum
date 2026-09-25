<script setup lang="ts">
import { PanelRightClose, PanelRightOpen } from "@lucide/vue";

import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";
import UiIconButton from "../../shared/ui/UiIconButton.vue";
import WorkspaceNavigationControls from "./WorkspaceNavigationControls.vue";

defineProps<{
  workspace: WorkspaceDefinition;
  sidebarPinned: boolean;
  sidebarWidth: number;
  rightSidebarOpen: boolean;
}>();
const emit = defineEmits<{
  "toggle-sidebar": [];
  "toggle-right-sidebar": [];
  "preview-sidebar": [];
  "cancel-sidebar-preview": [];
}>();
</script>

<template>
  <header
    class="@container relative h-(--knovum-window-chrome-height) shrink-0 material-toolbar [container-name:workspace-toolbar]"
  >
    <div data-tauri-drag-region class="absolute inset-0" />
    <div
      class="workspace-toolbar-content relative grid h-full"
      :style="{
        gridTemplateColumns: `${sidebarWidth}px minmax(0, 1fr)`,
      }"
    >
      <div
        class="flex min-w-0 items-center justify-between gap-1 pr-1 pl-(--knovum-window-navigation-control-offset)"
      >
        <WorkspaceNavigationControls
          :sidebar-pinned="sidebarPinned"
          @toggle-sidebar="emit('toggle-sidebar')"
          @preview-sidebar="emit('preview-sidebar')"
          @cancel-sidebar-preview="emit('cancel-sidebar-preview')"
        />
        <component :is="workspace.toolbarLeading" class="shrink-0" />
      </div>
      <div class="flex min-w-0 items-center gap-1 pr-3">
        <component :is="workspace.toolbar" class="min-w-0 flex-1" />
        <UiIconButton
          id="workspace-right-sidebar-toggle"
          class="shrink-0"
          :label="$t('shell.toggleRightSidebar')"
          aria-controls="spaces-right-sidebar"
          :aria-expanded="rightSidebarOpen"
          shortcut="$mod+Shift+B"
          :active="rightSidebarOpen"
          :disabled="!workspace.rightSidebarAvailable.value"
          size="sm"
          @click="emit('toggle-right-sidebar')"
        >
          <component
            :is="rightSidebarOpen ? PanelRightClose : PanelRightOpen"
            class="size-(--knovum-icon-size-sm)"
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
}
.workspace-toolbar-content :deep(button),
.workspace-toolbar-content :deep(input),
.workspace-toolbar-content :deep(select),
.workspace-toolbar-content :deep(textarea),
.workspace-toolbar-content :deep(a) {
  pointer-events: auto;
}
</style>

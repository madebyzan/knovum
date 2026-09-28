<script setup lang="ts">
import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";

defineProps<{ workspace: WorkspaceDefinition }>();
</script>

<template>
  <div
    class="relative h-full min-h-0 w-full min-w-0 overflow-hidden rounded-xl border border-(--knovum-window-border-color) bg-surface-panel shadow-workspace select-none"
  >
    <div class="workspace-right-sidebar-content absolute inset-y-0 right-0 w-full min-w-0">
      <component :is="workspace.rightSidebar" />
    </div>
  </div>
</template>

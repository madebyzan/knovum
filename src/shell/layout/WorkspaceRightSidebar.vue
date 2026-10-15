<script setup lang="ts">
import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";

defineProps<{ workspace: WorkspaceDefinition }>();
</script>

<template>
  <!--
    The pane is anchored to its own right edge at its own width, not stretched to whatever the pane
    currently measures. The pane takes its new width in one step, so the panel keeps its geometry while it
    slides itself in or out, and on the way out it is free to travel over the work area instead of being
    clipped to nothing on the first frame.
  -->
  <div class="absolute inset-y-0 right-0 w-(--workspace-right-sidebar-width) pr-2 pb-2">
    <div
      class="relative h-full min-h-0 w-full overflow-hidden rounded-xl border border-(--knovum-window-border-color) bg-surface-panel shadow-workspace select-none"
    >
      <component :is="workspace.rightSidebar" />
    </div>
  </div>
</template>

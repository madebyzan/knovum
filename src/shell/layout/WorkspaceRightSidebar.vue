<script setup lang="ts">
import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";

defineProps<{ workspace: WorkspaceDefinition }>();
</script>

<template>
  <!--
    Fixed at its own width and anchored to the slot's leading edge, so it rides that edge in from the window
    edge as the slot opens and never reflows its content while it does.
  -->
  <div
    class="absolute inset-y-0 left-0 w-(--workspace-right-sidebar-width) pr-(--knovum-workspace-gutter) pb-(--knovum-workspace-gutter)"
  >
    <div
      class="relative h-full min-h-0 w-full overflow-hidden rounded-xl border border-(--knovum-window-border-color) bg-surface-panel shadow-workspace select-none"
    >
      <component :is="workspace.rightSidebar" />
    </div>
  </div>
</template>

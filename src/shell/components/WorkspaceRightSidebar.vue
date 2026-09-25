<script setup lang="ts">
import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";

defineProps<{ workspace: WorkspaceDefinition }>();
</script>

<template>
  <div class="flex h-full min-w-0 flex-col items-stretch overflow-hidden select-none">
    <div class="flex min-h-0 flex-1 items-start overflow-hidden select-text">
      <KeepAlive :max="4">
        <component :is="workspace.rightSidebar" />
      </KeepAlive>
    </div>
  </div>
</template>

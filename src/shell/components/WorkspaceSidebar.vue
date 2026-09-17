<script setup lang="ts">
import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";

defineProps<{ workspace: WorkspaceDefinition }>();
</script>

<template>
  <aside class="flex h-full min-w-0 flex-col overflow-hidden material-navigation">
    <div class="min-h-0 flex-1 pt-2">
      <component :is="workspace.sidebar" />
    </div>
  </aside>
</template>

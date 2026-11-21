<script setup lang="ts">
import SpaceConversation from "../../conversation/presentation/SpaceConversation.vue";
</script>

<template>
  <aside
    class="pointer-events-auto flex h-full w-full min-w-0 flex-col overflow-hidden bg-surface-panel select-none"
  >
    <div class="min-h-0 flex-1 overflow-hidden select-text">
      <SpaceConversation class="h-full" />
    </div>
  </aside>
</template>

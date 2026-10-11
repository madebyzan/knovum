<script setup lang="ts">
import { useSpaceStore } from "../../session/space-store";
import SpaceSourcePeek from "../preview/SpaceSourcePeek.vue";
import SpaceContextHeader from "../workspace/SpaceContextHeader.vue";

const store = useSpaceStore();
</script>

<template>
  <aside
    v-if="store.sourcePeek"
    class="absolute inset-y-0 right-0 z-20 flex w-80 max-w-full flex-col border-l border-stroke-divider bg-surface-panel shadow-popover"
  >
    <SpaceContextHeader :title="store.sourcePeek.nodeTitle" closable @close="store.closeSource()" />
    <div class="min-h-0 flex-1"><SpaceSourcePeek /></div>
  </aside>
</template>

<script setup lang="ts">
import SpaceSourceReader from "../../reading/presentation/SpaceSourceReader.vue";
import { useSpaceStore } from "../../session/space-store";

const store = useSpaceStore();
</script>

<template>
  <div
    v-if="store.readingTarget"
    class="flex h-full w-[44%] max-w-[640px] min-w-[360px] shrink-0 flex-col border-l border-(--knovum-window-border-color) bg-surface-panel"
  >
    <SpaceSourceReader />
  </div>
</template>

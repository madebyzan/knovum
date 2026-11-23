<script setup lang="ts">
import SpaceSourceReader from "../../reading/presentation/SpaceSourceReader.vue";
import { useSpaceStore } from "../../session/space-store";

const store = useSpaceStore();
</script>

<template>
  <div
    v-if="store.readingTarget"
    :style="{ width: `${store.readerWidth}%` }"
    class="flex h-full max-w-160 min-w-90 shrink-0 flex-col border-l border-(--knovum-window-border-color) bg-surface-panel"
  >
    <SpaceSourceReader host="document" />
  </div>
</template>

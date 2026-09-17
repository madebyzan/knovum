<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed } from "vue";

const props = defineProps<{ path: string; title: string }>();
const source = computed(() => convertFileSrc(props.path));
</script>

<template>
  <div class="h-full overflow-hidden bg-canvas">
    <img
      :src="source"
      :alt="title"
      class="block size-full object-contain"
      draggable="false"
      loading="lazy"
      decoding="async"
    />
  </div>
</template>

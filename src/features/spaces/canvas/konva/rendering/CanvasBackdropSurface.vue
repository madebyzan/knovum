<script setup lang="ts">
import { computed } from "vue";

import { getCanvasBackdropAsset, type VisibleCanvasBackdrop } from "../../backdrops/backdrop-assets";

/**
 * Viewport-anchored curated backdrop sitting behind the SVG pattern and Konva stage.
 * Keeping it static relative to the host prevents re-rasterizing during camera pans and zooms,
 * while the theme-aware veil preserves foreground card, text, and relation contrast in both modes.
 */
const props = defineProps<{
  backdrop: VisibleCanvasBackdrop;
}>();

const source = computed(() => getCanvasBackdropAsset(props.backdrop));
</script>

<template>
  <div aria-hidden="true" class="pointer-events-none absolute inset-0 z-0 overflow-hidden select-none">
    <img :src="source" alt="" decoding="async" draggable="false" class="h-full w-full object-cover" />
    <div class="absolute inset-0" style="background-color: var(--knovum-space-backdrop-veil)" />
  </div>
</template>

<script setup lang="ts">
import { computed, type CSSProperties } from "vue";

import { rotatedImageSize, type ImageCanvasSize } from "../../../image-editing/canvas-image-renderer";
import type { ImageEdit } from "../../../image-editing/image-edit";

const props = defineProps<{
  position?: { x: number; y: number };
  size: ImageCanvasSize;
  naturalSize: { width: number; height: number };
  edit: ImageEdit;
  diameter: number;
}>();
const style = computed<CSSProperties>(() => {
  const cropWidth = props.naturalSize.width * props.edit.crop.width;
  const cropHeight = props.naturalSize.height * props.edit.crop.height;
  const rotated = rotatedImageSize(cropWidth, cropHeight, props.edit.rotation);
  const scale = Math.min(props.size.width / rotated.width, props.size.height / rotated.height);
  const brush = Math.max(
    6,
    Math.min(props.naturalSize.width, props.naturalSize.height) * scale * props.diameter,
  );
  return {
    left: `${props.position?.x ?? 0}px`,
    top: `${props.position?.y ?? 0}px`,
    width: `${brush}px`,
    height: `${brush}px`,
  };
});
</script>

<template>
  <span
    v-if="position && naturalSize.width && naturalSize.height"
    class="pointer-events-none absolute z-30 -translate-x-1/2 -translate-y-1/2 rounded-full border border-stroke-contrast bg-surface-raised/35 shadow-control"
    :style="style"
  />
</template>

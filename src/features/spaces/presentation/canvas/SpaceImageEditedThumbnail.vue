<script setup lang="ts">
import { useResizeObserver } from "@vueuse/core";
import { onBeforeUnmount, onMounted, useTemplateRef, watch } from "vue";

import { renderEditedImage } from "../../image-editing/canvas-image-renderer";
import type { ImageEdit } from "../../image-editing/image-edit";
import { versionedMediaSource } from "../media/element";

const props = defineProps<{ path: string; revision?: string; edit: ImageEdit }>();
const canvas = useTemplateRef<HTMLCanvasElement>("canvas");
const image = new Image();
image.crossOrigin = "anonymous";

function draw() {
  const element = canvas.value;
  const context = element?.getContext("2d", { willReadFrequently: true });
  if (!context || !element || !element.clientWidth || !element.clientHeight || !image.complete || !image.naturalWidth) return;
  renderEditedImage(context, image, {
    width: element.clientWidth,
    height: element.clientHeight,
    pixelRatio: element.width / Math.max(1, element.clientWidth),
  }, props.edit);
}

function loaded() {
  draw();
}

image.addEventListener("load", loaded);
watch(
  () => [props.path, props.revision] as const,
  ([path, revision]) => {
    const element = canvas.value;
    element?.getContext("2d")?.clearRect(0, 0, element.width, element.height);
    const source = versionedMediaSource(path, revision);
    if (source) image.src = source;
  },
  { immediate: true },
);
watch(() => props.edit, draw, { deep: true });
useResizeObserver(canvas, ([entry]) => {
  const box = entry?.contentRect;
  if (!canvas.value || !box) return;
  const ratio = Math.min(window.devicePixelRatio || 1, 2);
  canvas.value.width = Math.max(1, Math.round(box.width * ratio));
  canvas.value.height = Math.max(1, Math.round(box.height * ratio));
  draw();
});
onMounted(draw);
onBeforeUnmount(() => image.removeEventListener("load", loaded));
</script>

<template>
  <canvas ref="canvas" class="size-full" aria-hidden="true" />
</template>

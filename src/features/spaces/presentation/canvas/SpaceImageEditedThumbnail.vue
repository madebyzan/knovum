<script setup lang="ts">
import { error as logError } from "@tauri-apps/plugin-log";
import { useEventListener, useResizeObserver } from "@vueuse/core";
import { onMounted, useTemplateRef, watch } from "vue";

import { paintImageBitmap } from "../../image-editing/canvas-image-renderer";
import type { ImageEdit } from "../../image-editing/image-edit";
import { versionedMediaSource } from "../../media/media-element-source";
import { useCanvasImageRaster } from "./nodes/use-canvas-image-raster";

const props = defineProps<{ path: string; revision?: string | undefined; edit: ImageEdit }>();
const canvas = useTemplateRef<HTMLCanvasElement>("canvas");
const image = new Image();
const raster = useCanvasImageRaster({
  image,
  size: () => ({
    width: canvas.value!.clientWidth,
    height: canvas.value!.clientHeight,
    pixelRatio: Math.min(window.devicePixelRatio, 2),
  }),
  edit: () => props.edit,
  ready: () =>
    !!canvas.value?.clientWidth && !!canvas.value.clientHeight && image.complete && !!image.naturalWidth,
  active: () => false,
  paint: draw,
  error: (error) => {
    void logError(`Image thumbnail failed: ${String(error)}`);
  },
});
image.crossOrigin = "anonymous";

function draw() {
  const context = canvas.value?.getContext("2d");
  const bitmap = raster.bitmap.value;
  if (context && bitmap) paintImageBitmap(context, bitmap);
}

useEventListener(image, "load", raster.update);
watch(
  () => [props.path, props.revision] as const,
  ([path, revision]) => {
    raster.clear();
    const element = canvas.value;
    element?.getContext("2d")?.clearRect(0, 0, element.width, element.height);
    const source = versionedMediaSource(path, revision);
    if (source) image.src = source;
  },
  { immediate: true },
);
watch(() => props.edit, raster.update, { deep: true });
useResizeObserver(canvas, ([entry]) => {
  const box = entry?.contentRect;
  if (!canvas.value || !box) return;
  raster.update();
});
onMounted(raster.update);
</script>

<template>
  <canvas ref="canvas" class="size-full" aria-hidden="true" />
</template>

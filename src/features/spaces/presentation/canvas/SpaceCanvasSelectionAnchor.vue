<script setup lang="ts">
import { useElementSize } from "@vueuse/core";
import { computed, useTemplateRef } from "vue";

import type { CanvasPoint, CanvasRect } from "../../canvas/contract/canvas-editor-port";

const props = defineProps<{
  bounds?: CanvasRect | undefined;
  canvasToViewportPoint(point: CanvasPoint): CanvasPoint;
  safeArea: { top: number; right: number; bottom: number; left: number };
  align?: "center" | "start";
  gap?: number;
}>();
const host = useTemplateRef<HTMLElement>("host");
const toolbar = useTemplateRef<HTMLElement>("toolbar");
const hostSize = useElementSize(host);
const toolbarSize = useElementSize(toolbar);

const geometry = computed(() => {
  if (!props.bounds || !toolbarSize.width.value || !toolbarSize.height.value) return undefined;
  const start = props.canvasToViewportPoint(props.bounds);
  const end = props.canvasToViewportPoint({
    x: props.bounds.x + props.bounds.width,
    y: props.bounds.y + props.bounds.height,
  });
  return { start, end };
});

const position = computed(() => {
  if (!geometry.value) return undefined;
  const { start, end } = geometry.value;
  const width = toolbarSize.width.value;
  const startAligned = props.align === "start";
  const left = startAligned
    ? clamp(start.x, props.safeArea.left, hostSize.width.value - props.safeArea.right - width)
    : clamp(
        (start.x + end.x) / 2,
        props.safeArea.left + width / 2,
        hostSize.width.value - props.safeArea.right - width / 2,
      );
  const gap = props.gap ?? 34;
  const fitsAbove = start.y - gap - toolbarSize.height.value >= props.safeArea.top;
  const preferredTop = fitsAbove ? start.y - gap - toolbarSize.height.value : end.y + gap;
  const top = clamp(
    preferredTop,
    props.safeArea.top,
    hostSize.height.value - props.safeArea.bottom - toolbarSize.height.value,
  );
  return { left: `${left}px`, top: `${top}px` };
});

function clamp(value: number, minimum: number, maximum: number) {
  return Math.min(Math.max(value, minimum), Math.max(minimum, maximum));
}
</script>

<template>
  <div ref="host" class="pointer-events-none absolute inset-0">
    <div
      v-if="bounds"
      ref="toolbar"
      class="pointer-events-auto absolute w-max transition-opacity duration-100 ease-standard motion-reduce:transition-none"
      :class="[position ? 'visible' : 'invisible', align === 'start' ? '' : '-translate-x-1/2']"
      :style="position"
    >
      <slot :available-width="Math.max(0, hostSize.width.value - safeArea.left - safeArea.right)" />
    </div>
  </div>
</template>

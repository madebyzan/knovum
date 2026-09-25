<script setup lang="ts">
import { useElementSize } from "@vueuse/core";
import { computed, useTemplateRef, watch } from "vue";

import type { CanvasPoint, CanvasRect } from "../../canvas/contract/canvas-editor-port";
import { useCanvasEditorSession } from "../../canvas/editor-session";
import { placeSelectionSurface } from "./selection-surface-placement";
import { useCanvasSurfaceObstacles } from "./use-canvas-surface-obstacles";

const props = defineProps<{
  bounds?: CanvasRect | undefined;
  canvasToViewportPoint(point: CanvasPoint): CanvasPoint;
  safeArea: { top: number; right: number; bottom: number; left: number };
  gap?: number;
  kind: "actions" | "compact-editor" | "inspector";
}>();
const emit = defineEmits<{ ready: [] }>();
const editorSession = useCanvasEditorSession();
const canvasObstacles = useCanvasSurfaceObstacles();
const host = useTemplateRef<HTMLElement>("host");
const toolbar = useTemplateRef<HTMLElement>("toolbar");
const hostSize = useElementSize(host);
const toolbarSize = useElementSize(toolbar);

const geometry = computed(() => {
  if (
    !props.bounds ||
    !editorSession.viewport.value ||
    !hostSize.width.value ||
    !hostSize.height.value ||
    !toolbarSize.width.value ||
    !toolbarSize.height.value
  )
    return undefined;
  const start = props.canvasToViewportPoint(props.bounds);
  const end = props.canvasToViewportPoint({
    x: props.bounds.x + props.bounds.width,
    y: props.bounds.y + props.bounds.height,
  });
  return { start, end };
});

const position = computed(() => {
  const bounds = props.bounds;
  if (!geometry.value || !bounds) return undefined;
  const { start, end } = geometry.value;
  const reach =
    Math.max(hostSize.width.value, hostSize.height.value) / (editorSession.viewport.value?.zoom ?? 1);
  const obstacles = canvasObstacles.near(bounds, reach).map(({ left, top, right, bottom }) => {
    const first = props.canvasToViewportPoint({ x: left, y: top });
    const last = props.canvasToViewportPoint({ x: right, y: bottom });
    return { left: first.x, top: first.y, right: last.x, bottom: last.y };
  });
  const placed = placeSelectionSurface({
    selection: { left: start.x, top: start.y, right: end.x, bottom: end.y },
    obstacles,
    viewport: { width: hostSize.width.value, height: hostSize.height.value },
    insets: props.safeArea,
    surface: {
      width: toolbarSize.width.value,
      height: Math.max(toolbar.value?.scrollHeight ?? 0, toolbarSize.height.value),
    },
    preferredAxis: props.kind === "inspector" ? "horizontal" : "vertical",
    gap: props.gap ?? 34,
  });
  return {
    left: `${placed.left}px`,
    top: `${placed.top}px`,
    maxHeight: `${placed.maxHeight}px`,
  };
});
watch(
  position,
  (value, previous) => {
    if (value && !previous) emit("ready");
  },
  { flush: "post" },
);
</script>

<template>
  <div ref="host" class="pointer-events-none absolute inset-0">
    <div
      v-if="bounds"
      ref="toolbar"
      class="pointer-events-auto absolute w-max transition-opacity duration-100 ease-standard motion-reduce:transition-none"
      :class="[
        position ? 'visible' : 'invisible',
        kind === 'actions' ? '' : 'overflow-x-hidden overflow-y-auto overscroll-contain',
      ]"
      :style="position"
    >
      <slot
        :available-width="Math.max(0, hostSize.width.value - safeArea.left - safeArea.right)"
        :available-height="position ? Number.parseFloat(position.maxHeight) : undefined"
      />
    </div>
  </div>
</template>

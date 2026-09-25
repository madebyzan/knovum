<script setup lang="ts">
import { useElementSize } from "@vueuse/core";
import { computed, shallowRef, useTemplateRef, watch } from "vue";

import type { CanvasPoint, CanvasRect } from "../../canvas/contract/canvas-editor-port";
import { useCanvasEditorSession } from "../../canvas/editor-session";
import { type SurfaceRect, type SurfaceSide } from "./selection-surface-geometry";
import { placeSelectionSurface } from "./selection-surface-placement";
import { useSurfaceTriggerOpen } from "./use-surface-trigger-open";

const props = defineProps<{
  bounds?: CanvasRect | undefined;
  members?: readonly CanvasRect[] | undefined;
  canvasToViewportPoint(point: CanvasPoint): CanvasPoint;
  safeArea: { top: number; right: number; bottom: number; left: number };
  reserved: readonly SurfaceRect[];
  selectionKey: string;
  gap?: number;
  kind: "actions" | "compact-editor" | "inspector";
}>();
const emit = defineEmits<{ ready: [] }>();
const editorSession = useCanvasEditorSession();
const host = useTemplateRef<HTMLElement>("host");
const toolbar = useTemplateRef<HTMLElement>("toolbar");
const hostSize = useElementSize(host);
const toolbarSize = useElementSize(toolbar);
const previousSide = shallowRef<SurfaceSide>();
const transientOpen = useSurfaceTriggerOpen(toolbar);

watch(
  () => [props.selectionKey, props.kind],
  () => (previousSide.value = undefined),
  { flush: "sync" },
);

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
  const members = props.members?.map((member) => {
    const first = props.canvasToViewportPoint(member);
    const last = props.canvasToViewportPoint({ x: member.x + member.width, y: member.y + member.height });
    return { left: first.x, top: first.y, right: last.x, bottom: last.y };
  });
  return { start, end, members };
});

const placement = computed(() => {
  if (!geometry.value) return undefined;
  const { start, end } = geometry.value;
  return placeSelectionSurface({
    selection: { left: start.x, top: start.y, right: end.x, bottom: end.y },
    members: geometry.value.members ?? [],
    reserved: props.reserved,
    viewport: { width: hostSize.width.value, height: hostSize.height.value },
    insets: props.safeArea,
    surface: {
      width: toolbarSize.width.value,
      height: Math.max(toolbar.value?.scrollHeight ?? 0, toolbarSize.height.value),
    },
    kind: props.kind,
    gap: props.gap ?? 34,
    lockSide: transientOpen.value,
    ...(previousSide.value ? { previousSide: previousSide.value } : {}),
  });
});
const position = computed(() => {
  const placed = placement.value;
  if (!placed) return undefined;
  return {
    left: `${placed.left}px`,
    top: `${placed.top}px`,
    maxHeight: `${placed.maxHeight}px`,
  };
});
watch(
  placement,
  (value) => {
    if (value) previousSide.value = value.side;
  },
  { flush: "post" },
);
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
      :data-side="placement?.side"
    >
      <slot
        :available-width="Math.max(0, hostSize.width.value - safeArea.left - safeArea.right)"
        :available-height="position ? Number.parseFloat(position.maxHeight) : undefined"
      />
    </div>
  </div>
</template>

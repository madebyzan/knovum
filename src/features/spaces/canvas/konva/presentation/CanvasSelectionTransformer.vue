<script setup lang="ts">
import type Konva from "konva";
import { computed, nextTick, useTemplateRef, watch } from "vue";
import { Transformer as KonvaTransformer } from "vue-konva/core";
import type { VueKonvaRef } from "vue-konva/core";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Transformer";

import type { CanvasNodeFrame } from "../../contract/canvas-document";
import { captureCanvasNodeRefs } from "../interaction/canvas-node-access";
import type { CanvasSnapGuide } from "../interaction/canvas-snapping";
import {
  createCanvasResizeSnapIndex,
  resizeSnapCoordinate,
  resizeSnapGuides,
  type CanvasResizeSnapIndex,
} from "../interaction/resize-snap-guides";
import { canvasResizeAnchors, isProportionalResizeAnchor } from "../selection/canvas-resize-anchors";
import { constrainSelectionTransform } from "../selection/selection-transform-geometry";
import type { CanvasSelectionTransformPolicy } from "../selection/selection-transform-policy";
import { canvasChromeMetrics } from "./canvas-chrome-metrics";
import type { CanvasPalette } from "./use-canvas-palette";

const props = defineProps<{
  interactive: boolean;
  palette: CanvasPalette;
  policy: CanvasSelectionTransformPolicy;
  selectedFrames: readonly CanvasNodeFrame[];
  snapFrames: readonly CanvasNodeFrame[];
  stage: Konva.Stage;
  visible: boolean;
  zoom: number;
}>();
const emit = defineEmits<{
  resizeSnap: [guides: readonly CanvasSnapGuide[]];
  transform: [proportionalAnchor?: string];
  transformEnd: [];
  transformStart: [];
}>();
const transformer = useTemplateRef<VueKonvaRef<Konva.Transformer>>("transformer");
const metrics = computed(() => canvasChromeMetrics(props.zoom).resize);
let activeResizeAnchor: string | undefined;
let resizeSnapIndex: CanvasResizeSnapIndex | undefined;
const enabledResizeAnchors = computed(() => {
  return props.interactive ? canvasResizeAnchors(props.policy.resizeMode) : [];
});
const selectedFrameGeometry = computed(() =>
  props.selectedFrames.flatMap(({ x, y, width, height, rotation }) => [x, y, width, height, rotation]),
);

watch(
  [
    () => props.policy.nodeIds.join("\u0000"),
    () => props.stage,
    selectedFrameGeometry,
    () => enabledResizeAnchors.value.join("\u0000"),
    () => props.policy.resizeMode,
  ],
  async () => {
    await nextTick();
    const instance = transformer.value?.getNode();
    if (!instance) return;
    instance.nodes([...captureCanvasNodeRefs(props.stage, props.policy.nodeIds).values()]);
    instance.resizeEnabled(props.interactive && props.policy.resizeMode !== "fixed");
    instance.keepRatio(props.policy.resizeMode === "proportional" && !props.policy.contentAspect);
    instance.enabledAnchors(enabledResizeAnchors.value);
    instance.forceUpdate();
  },
  { immediate: true },
);

const constrainResize: NonNullable<Konva.TransformerConfig["boundBoxFunc"]> = (current, next) => {
  return constrainSelectionTransform(
    current,
    next,
    transformer.value?.getNode()?.getActiveAnchor() ?? null,
    props.policy,
    props.stage.getAbsoluteScale(),
  );
};

const snapAnchor: NonNullable<Konva.TransformerConfig["anchorDragBoundFunc"]> = (_, next) => {
  const activeAnchor = transformer.value?.getNode()?.getActiveAnchor();
  if (!activeAnchor || !resizeSnapIndex) return next;
  const transform = props.stage.getAbsoluteTransform();
  const x = resizeSnapCoordinate({
    activeAnchor,
    axis: "x",
    index: resizeSnapIndex,
    value: next.x,
  });
  const y = resizeSnapCoordinate({
    activeAnchor,
    axis: "y",
    index: resizeSnapIndex,
    value: next.y,
  });
  const snapped = { x: x ?? next.x, y: y ?? next.y };
  emit("resizeSnap", resizeSnapGuides({ index: resizeSnapIndex, snapped, transform, x, y }));
  return snapped;
};

const styleResizeAnchor: NonNullable<Konva.TransformerConfig["anchorStyleFunc"]> = (anchor) => {
  anchor.hitStrokeWidth(metrics.value.hitStrokeWidth);
  anchor.visible(isResizeAnchorVisible(anchor));
};
function isResizeAnchorVisible(anchor: Konva.Rect) {
  const enabled = enabledResizeAnchors.value.some((name) => anchor.hasName(name));
  return enabled && (activeResizeAnchor === undefined || anchor.hasName(activeResizeAnchor));
}

function applyResizeAnchorVisibility() {
  const instance = transformer.value?.getNode();
  if (!instance) return;
  for (const anchor of instance.find("._anchor")) {
    anchor.visible(isResizeAnchorVisible(anchor));
  }
  instance.getLayer()?.batchDraw();
}

function beginTransform() {
  activeResizeAnchor = transformer.value?.getNode()?.getActiveAnchor() ?? undefined;
  resizeSnapIndex = createCanvasResizeSnapIndex(props.snapFrames, props.stage.getAbsoluteTransform());
  applyResizeAnchorVisibility();
  emit("transformStart");
}

function updateTransform() {
  activeResizeAnchor ??= transformer.value?.getNode()?.getActiveAnchor() ?? undefined;
  const proportionalAnchor = isProportionalResizeAnchor(props.policy.resizeMode, activeResizeAnchor)
    ? activeResizeAnchor
    : undefined;
  emit("transform", proportionalAnchor);
}

function finishTransform() {
  activeResizeAnchor = undefined;
  resizeSnapIndex = undefined;
  applyResizeAnchorVisibility();
  emit("resizeSnap", []);
  emit("transformEnd");
}
</script>

<template>
  <KonvaTransformer
    ref="transformer"
    :config="{
      rotateEnabled: false,
      visible,
      listening: interactive && visible,
      flipEnabled: false,
      borderEnabled: policy.nodeIds.length > 1 || policy.resizeMode === 'width-and-proportional',
      borderStroke: palette.accent,
      borderStrokeWidth: 1 / zoom,
      anchorFill: palette.accent,
      anchorStroke: palette.surface,
      anchorStrokeWidth: metrics.strokeWidth,
      anchorSize: metrics.size,
      anchorCornerRadius: metrics.cornerRadius,
      anchorStyleFunc: styleResizeAnchor,
      padding: 0,
      boundBoxFunc: constrainResize,
      anchorDragBoundFunc: snapAnchor,
    }"
    @transformstart="beginTransform"
    @transform="updateTransform"
    @transformend="finishTransform"
  />
</template>

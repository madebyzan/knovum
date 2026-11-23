<script setup lang="ts">
import { onScopeDispose, useId, useTemplateRef, watch } from "vue";

import type { CanvasBackgroundPattern } from "../../../../../shared/contracts/generated/CanvasBackgroundPattern";
import type { CanvasViewport } from "../../contracts/canvas-editor-port";
import type { CanvasPalette } from "../../use-canvas-palette";
import type { CanvasCamera } from "../viewport/canvas-camera";

const GRID_SPACING = 25;
const DOT_RADIUS = 1.1;

type VisiblePattern = Exclude<CanvasBackgroundPattern, "none">;

const props = defineProps<{
  camera: CanvasCamera;
  palette: CanvasPalette;
  pattern: VisiblePattern;
  surface: { height: number; width: number };
  zoom: number;
}>();
const patternNode = useTemplateRef<SVGPatternElement>("patternNode");
const dot = useTemplateRef<SVGCircleElement>("dot");
const gridLines = useTemplateRef<SVGPathElement>("gridLines");
const patternId = `canvas-grid-${useId()}`;

/** Phase that keeps pattern intersections on every world multiple of the spacing, for any translation sign. */
function phase(translation: number, cell: number, mode: VisiblePattern) {
  const offset = mode === "dots" ? translation - cell / 2 : translation;
  return ((offset % cell) + cell) % cell;
}

function applyPhase(node: SVGPatternElement, camera: CanvasViewport, mode: VisiblePattern) {
  const cell = GRID_SPACING * camera.zoom;
  node.setAttribute(
    "patternTransform",
    `translate(${phase(camera.x, cell, mode)} ${phase(camera.y, cell, mode)})`,
  );
}

/** The cell follows the zoom while dot radius and line stroke stay constant in screen pixels. */
function applyCell(node: SVGPatternElement, camera: CanvasViewport, mode: VisiblePattern) {
  const cell = GRID_SPACING * camera.zoom;
  node.setAttribute("width", String(cell));
  node.setAttribute("height", String(cell));
  if (mode === "dots" && dot.value) {
    dot.value.setAttribute("cx", String(cell / 2));
    dot.value.setAttribute("cy", String(cell / 2));
  } else if (mode === "grid" && gridLines.value) {
    gridLines.value.setAttribute("d", `M ${cell} 0 L 0 0 0 ${cell}`);
  }
  applyPhase(node, camera, mode);
}

watch(
  [patternNode, dot, gridLines, () => props.zoom, () => props.pattern],
  ([node]) => node && applyCell(node, props.camera.get(), props.pattern),
  { flush: "post", immediate: true },
);
/**
 * The background pattern is an SVG pattern, written imperatively per camera frame like every other
 * camera-anchored surface. A CSS gradient cannot hold this role: the engine pixel-snaps background
 * geometry, making it jitter against subpixel-rendered nodes. A pattern transform interpolates in
 * floating point and rasterizes one tile on its own compositor layer.
 */
onScopeDispose(
  props.camera.observeFrames(
    (camera) => patternNode.value && applyPhase(patternNode.value, camera, props.pattern),
  ),
);
</script>

<template>
  <svg
    aria-hidden="true"
    class="pointer-events-none absolute top-0 left-0 z-0 will-change-transform"
    :width="surface.width"
    :height="surface.height"
  >
    <defs>
      <pattern :id="patternId" ref="patternNode" patternUnits="userSpaceOnUse" width="25" height="25">
        <circle
          v-if="pattern === 'dots'"
          ref="dot"
          :r="DOT_RADIUS"
          :fill="palette.grid"
          cx="12.5"
          cy="12.5"
        />
        <path
          v-else
          ref="gridLines"
          d="M 25 0 L 0 0 0 25"
          fill="none"
          :stroke="palette.gridLine"
          stroke-width="1"
        />
      </pattern>
    </defs>
    <rect width="100%" height="100%" :fill="`url(#${patternId})`" />
  </svg>
</template>

<script setup lang="ts">
import { useId, useTemplateRef, watch } from "vue";

import type { CanvasViewport } from "../../contracts/canvas-editor-port";
import type { CanvasCamera } from "../viewport/canvas-camera";
import type { CanvasPalette } from "./use-canvas-palette";

const GRID_SPACING = 25;
const DOT_RADIUS = 1.1;

const props = defineProps<{
  camera: CanvasCamera;
  palette: CanvasPalette;
  surface: { height: number; width: number };
  zoom: number;
}>();
const pattern = useTemplateRef<SVGPatternElement>("pattern");
const dot = useTemplateRef<SVGCircleElement>("dot");
const patternId = `canvas-grid-${useId()}`;

/** Phase that keeps a dot centred on every world multiple of the spacing, for any sign of translation. */
function phase(translation: number, cell: number) {
  return (((translation - cell / 2) % cell) + cell) % cell;
}

function applyPhase(node: SVGPatternElement, camera: CanvasViewport) {
  const cell = GRID_SPACING * camera.zoom;
  node.setAttribute("patternTransform", `translate(${phase(camera.x, cell)} ${phase(camera.y, cell)})`);
}

/** The cell follows the zoom, the dot does not: dots stay the same size on screen at every zoom level. */
function applyCell(node: SVGPatternElement, circle: SVGCircleElement, camera: CanvasViewport) {
  const cell = GRID_SPACING * camera.zoom;
  node.setAttribute("width", String(cell));
  node.setAttribute("height", String(cell));
  circle.setAttribute("cx", String(cell / 2));
  circle.setAttribute("cy", String(cell / 2));
  applyPhase(node, camera);
}

watch(
  [pattern, dot, () => props.zoom],
  ([node, circle]) => node && circle && applyCell(node, circle, props.camera.get()),
  { flush: "post", immediate: true },
);
props.camera.observeFrames((camera) => pattern.value && applyPhase(pattern.value, camera));
</script>

<template>
  <!--
    The dot grid is an SVG pattern, written imperatively per camera frame like every other camera-anchored
    surface. A CSS gradient background cannot hold this role: the engine pixel-snaps background geometry,
    so the grid would step a whole device pixel at a time while subpixel-rendered content moves smoothly -
    visible as the grid jittering against the nodes. A pattern transform is interpolated in floating point,
    and one tile is rasterized rather than a viewport-sized Konva layer being refilled per frame.

    It spans the stage surface rather than the host and sits on its own compositor layer, so a work-area
    edge moving under a sidebar clips and carries the grid without ever re-rasterizing it.
  -->
  <svg
    aria-hidden="true"
    class="pointer-events-none absolute top-0 left-0 z-0 will-change-transform"
    :width="surface.width"
    :height="surface.height"
  >
    <defs>
      <pattern :id="patternId" ref="pattern" patternUnits="userSpaceOnUse" width="25" height="25">
        <circle ref="dot" :r="DOT_RADIUS" :fill="palette.grid" cx="12.5" cy="12.5" />
      </pattern>
    </defs>
    <rect width="100%" height="100%" :fill="`url(#${patternId})`" />
  </svg>
</template>

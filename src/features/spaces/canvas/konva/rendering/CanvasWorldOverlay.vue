<script setup lang="ts">
import { useTemplateRef } from "vue";

import { isScrollableTarget } from "../../../../../foundation/dom/scrollable-target";
import type { CanvasCamera } from "../viewport/canvas-camera";
import { useCameraStyle, worldTransform } from "../viewport/use-camera-style";

/**
 * Product surfaces that live in canvas space (a generator's frame and composer) but take their
 * own input. The overlay is a sibling of the stage, never inside it: a press on a control here
 * must not also start a stage gesture, and must still reach the document so menus and popovers
 * dismiss as they do everywhere else. Keys and context menus stay here too, so typing a space is
 * not a pan and a right-click is not the canvas menu. Only the wheel is handed to the stage, so
 * the canvas pans and zooms over these surfaces the way it does over nodes.
 */
const props = defineProps<{
  camera: CanvasCamera;
  renderingEnabled: boolean;
  stageContent: HTMLElement | undefined;
}>();
const world = useTemplateRef<HTMLElement>("world");
useCameraStyle(props.camera, world, worldTransform);

function forwardWheel(event: WheelEvent) {
  if (!props.stageContent || isScrollableTarget(event.target)) return;
  event.preventDefault();
  props.stageContent.dispatchEvent(new WheelEvent(event.type, event));
}
</script>

<template>
  <div
    :inert="!renderingEnabled"
    data-thumbnail-exclude
    class="pointer-events-none absolute inset-0 z-[18] overflow-hidden"
    @wheel="forwardWheel"
    @keydown.stop
    @contextmenu.stop
  >
    <div ref="world" class="absolute top-0 left-0 origin-top-left will-change-transform">
      <slot />
    </div>
  </div>
</template>

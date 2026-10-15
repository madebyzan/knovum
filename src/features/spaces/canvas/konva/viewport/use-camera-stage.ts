import type Konva from "konva";
import { Konva as konvaGlobal } from "konva/lib/Global";
import { onScopeDispose, watch, type Ref } from "vue";

import type { CanvasViewport } from "../../contracts/canvas-editor-port";
import type { CanvasCamera } from "./canvas-camera";

function applyTransform(node: Konva.Stage, value: CanvasViewport) {
  node.position({ x: value.x, y: value.y });
  node.scale({ x: value.zoom, y: value.zoom });
}

/**
 * Applies the live camera to the Konva stage.
 *
 * Konva redraws every layer when a stage attribute changes, one frame after the change. The DOM content
 * overlay follows the camera in the same frame, so a batched redraw would leave the canvases a frame behind
 * the cards. While the camera moves this composable therefore owns the render loop: `autoDrawEnabled`
 * suppresses Konva's own scheduling and each frame draws the layers synchronously, right after the stage
 * transform is written. Non-listening layers skip their hit graph, which is why panning - where pointer
 * interaction is suspended - costs one scene draw per layer and nothing else.
 */
export function useCameraStage(camera: CanvasCamera, stage: Readonly<Ref<Konva.Stage | undefined>>) {
  let owningRenderLoop = false;

  function releaseRenderLoop() {
    if (!owningRenderLoop) return;
    owningRenderLoop = false;
    konvaGlobal.autoDrawEnabled = true;
  }

  function drawFrame(value: CanvasViewport) {
    const node = stage.value;
    if (!node) return;
    if (!owningRenderLoop) {
      owningRenderLoop = true;
      konvaGlobal.autoDrawEnabled = false;
    }
    applyTransform(node, value);
    for (const layer of node.getLayers()) layer.draw();
  }

  function drawSettled(value: CanvasViewport) {
    releaseRenderLoop();
    const node = stage.value;
    if (!node) return;
    applyTransform(node, value);
    for (const layer of node.getLayers()) layer.draw();
  }

  const stopFrames = camera.observeFrames(drawFrame);
  const stopSettled = camera.observeSettled(drawSettled);
  watch(stage, (node) => node && drawSettled(camera.get()), { immediate: true });
  onScopeDispose(() => {
    stopFrames();
    stopSettled();
    releaseRenderLoop();
  });
}

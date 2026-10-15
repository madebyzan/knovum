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
 * transform is written.
 *
 * A pan gesture additionally takes the stage out of hit testing. Konva resolves a hit by reading pixels back
 * from each listening layer's hit canvas on every pointer move, then dispatches enter and leave events that
 * reach product state - work that is pure waste while the pointer is dragging the canvas itself, and that
 * also makes the frame pay a second draw per layer for the hit graph. A non-listening stage short-circuits
 * all of it, while the stage still receives the pointer events the gesture itself needs. Hit graphs are
 * restored and redrawn on settlement, which the gesture holds open until it ends.
 */
export function useCameraStage(
  camera: CanvasCamera,
  stage: Readonly<Ref<Konva.Stage | undefined>>,
  panning: Readonly<Ref<boolean>>,
) {
  let owningRenderLoop = false;
  let hitSuspended = false;

  function engage(node: Konva.Stage) {
    if (owningRenderLoop) return;
    owningRenderLoop = true;
    konvaGlobal.autoDrawEnabled = false;
    if (!panning.value) return;
    hitSuspended = true;
    node.listening(false);
  }

  function release() {
    owningRenderLoop = false;
    konvaGlobal.autoDrawEnabled = true;
    if (!hitSuspended) return;
    hitSuspended = false;
    stage.value?.listening(true);
  }

  function drawFrame(value: CanvasViewport) {
    const node = stage.value;
    if (!node) return;
    engage(node);
    applyTransform(node, value);
    for (const layer of node.getLayers()) {
      if (hitSuspended) layer.drawScene();
      else layer.draw();
    }
  }

  function drawSettled(value: CanvasViewport) {
    release();
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
    release();
  });
}

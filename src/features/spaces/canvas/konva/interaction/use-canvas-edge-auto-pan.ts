import { onBeforeUnmount, type Reactive } from "vue";

import type { CanvasPoint, CanvasViewport } from "../../contracts/canvas-editor-port";
import { edgeAutoPanVelocity } from "./edge-auto-pan";

interface EdgeAutoPanOptions {
  onPan(change: { canvasDelta: CanvasPoint; screenPoint: CanvasPoint }): void;
  setViewport(viewport: CanvasViewport): void;
  size: Reactive<{ width: number; height: number }>;
  viewport(): CanvasViewport;
}

export function useCanvasEdgeAutoPan(options: EdgeAutoPanOptions) {
  let frame: number | undefined;
  let pointer: CanvasPoint | undefined;
  let previousTime: number | undefined;

  function update(next: CanvasPoint) {
    pointer = next;
    if (frame === undefined) frame = requestAnimationFrame(tick);
  }

  function tick(time: number) {
    frame = undefined;
    if (!pointer) return;
    const velocity = edgeAutoPanVelocity(pointer, options.size);
    const elapsedSeconds = Math.min(32, previousTime === undefined ? 16 : time - previousTime) / 1000;
    previousTime = time;
    if (velocity.x === 0 && velocity.y === 0) return;
    const screenDelta = { x: velocity.x * elapsedSeconds, y: velocity.y * elapsedSeconds };
    const viewport = options.viewport();
    options.setViewport({ ...viewport, x: viewport.x - screenDelta.x, y: viewport.y - screenDelta.y });
    options.onPan({
      canvasDelta: { x: screenDelta.x / viewport.zoom, y: screenDelta.y / viewport.zoom },
      screenPoint: pointer,
    });
    frame = requestAnimationFrame(tick);
  }

  function stop() {
    pointer = undefined;
    previousTime = undefined;
    if (frame !== undefined) cancelAnimationFrame(frame);
    frame = undefined;
  }

  onBeforeUnmount(stop);
  return { stop, update };
}

import type { CanvasViewport } from "../../contracts/canvas-editor-port";
import { sameCanvasViewport } from "../document/canvas-document-mutations";

type CanvasCameraObserver = (camera: CanvasViewport) => void;

/**
 * Live camera of a mounted canvas surface.
 *
 * The camera is deliberately outside Vue reactivity: a pan or zoom gesture changes it many times per
 * second, and every consumer that must follow it at that rate paints imperatively from `observeFrames`.
 * Reactive consumers use the settled value instead, so moving the camera never re-renders the scene tree.
 */
export interface CanvasCamera {
  get(): CanvasViewport;
  set(camera: CanvasViewport): void;
  /** Projects a pending change and reports settlement immediately instead of waiting for the next frame. */
  settle(): void;
  /** Called once per animation frame while the camera moves. Subscribers apply the current value themselves. */
  observeFrames(observer: CanvasCameraObserver): () => void;
  /** Called on the first frame after the camera stops moving. */
  observeSettled(observer: CanvasCameraObserver): () => void;
  dispose(): void;
}

export function createCanvasCamera(initial: CanvasViewport): CanvasCamera {
  const frameObservers = new Set<CanvasCameraObserver>();
  const settledObservers = new Set<CanvasCameraObserver>();
  let current = initial;
  let projected = initial;
  let movedSinceSettlement = false;
  let projectionFrame: number | undefined;
  let settlementFrame: number | undefined;

  function cancelSettlement() {
    if (settlementFrame === undefined) return;
    cancelAnimationFrame(settlementFrame);
    settlementFrame = undefined;
  }

  function emitFrame() {
    if (projected === current) return;
    projected = current;
    movedSinceSettlement = true;
    for (const observer of frameObservers) observer(current);
  }

  function project() {
    projectionFrame = undefined;
    emitFrame();
    settlementFrame = requestAnimationFrame(reportSettlement);
  }

  function reportSettlement() {
    settlementFrame = undefined;
    if (!movedSinceSettlement) return;
    movedSinceSettlement = false;
    for (const observer of settledObservers) observer(current);
  }

  return {
    get: () => current,
    set(camera) {
      if (sameCanvasViewport(current, camera)) return;
      current = camera;
      cancelSettlement();
      if (projectionFrame === undefined) projectionFrame = requestAnimationFrame(project);
    },
    settle() {
      if (projectionFrame !== undefined) cancelAnimationFrame(projectionFrame);
      projectionFrame = undefined;
      cancelSettlement();
      emitFrame();
      reportSettlement();
    },
    observeFrames(observer) {
      frameObservers.add(observer);
      return () => frameObservers.delete(observer);
    },
    observeSettled(observer) {
      settledObservers.add(observer);
      return () => settledObservers.delete(observer);
    },
    dispose() {
      if (projectionFrame !== undefined) cancelAnimationFrame(projectionFrame);
      projectionFrame = undefined;
      cancelSettlement();
      frameObservers.clear();
      settledObservers.clear();
    },
  };
}

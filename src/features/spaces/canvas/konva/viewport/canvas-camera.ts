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
  /**
   * Holds settlement open until the returned release is called.
   *
   * Settlement is the expensive edge: it hands drawing back to the vendor, redraws every layer including
   * the hit graph, and lets the document adopt the camera. Without a hold, a pointer that rests for a
   * single frame - which a human hand does constantly mid-gesture - would pay all of that and then take it
   * back on the next move. A gesture therefore holds the camera from start to release.
   */
  hold(): () => void;
  /** Called once per animation frame while the camera moves. Subscribers apply the current value themselves. */
  observeFrames(observer: CanvasCameraObserver): () => void;
  /** Called on the first frame after the camera stops moving. */
  observeSettled(observer: CanvasCameraObserver): () => void;
  dispose(): void;
}

export function createCanvasCamera(initial: CanvasViewport): CanvasCamera {
  const frameObservers = new Set<CanvasCameraObserver>();
  const settledObservers = new Set<CanvasCameraObserver>();
  const holders = new Set<symbol>();
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
    if (holders.size > 0 || !movedSinceSettlement) return;
    movedSinceSettlement = false;
    for (const observer of settledObservers) observer(current);
  }

  function releaseHold() {
    if (holders.size > 0 || !movedSinceSettlement) return;
    if (projectionFrame !== undefined || settlementFrame !== undefined) return;
    settlementFrame = requestAnimationFrame(reportSettlement);
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
      holders.clear();
      emitFrame();
      reportSettlement();
    },
    hold() {
      const token = Symbol("canvas-camera-hold");
      holders.add(token);
      return () => {
        if (!holders.delete(token)) return;
        releaseHold();
      };
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
      holders.clear();
      frameObservers.clear();
      settledObservers.clear();
    },
  };
}

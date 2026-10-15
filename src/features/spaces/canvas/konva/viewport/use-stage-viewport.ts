import { TransitionPresets, transition, useElementSize, useWindowSize } from "@vueuse/core";
import type Konva from "konva";
import { computed, onBeforeUnmount, onScopeDispose, reactive, shallowRef, watch, type Ref } from "vue";

import { isScrollableTarget } from "../../../../../foundation/dom/scrollable-target";
import type { CanvasViewport } from "../../contracts/canvas-editor-port";
import { createCameraPan } from "./camera-pan";
import { createCanvasCamera } from "./canvas-camera";
import { useStageSurfaceSize } from "./use-stage-surface-size";
import { useViewportSpaceKey } from "./use-viewport-space-key";
import { normalizedWheelDelta, zoomViewportAt } from "./viewport-math";

interface StageViewportOptions {
  host: Readonly<Ref<HTMLElement | null>>;
  initialViewport: CanvasViewport;
  navigationEnabled(): boolean;
  /** Receives every projected camera frame, for screen-anchored product surfaces outside the stage. */
  project(viewport: CanvasViewport): void;
  /** Receives the camera once it stops moving, which is when the adapter document adopts it. */
  commit(viewport: CanvasViewport): void;
}

const ANIMATION_MS = 180;

/**
 * Camera ownership for one mounted stage.
 *
 * Gestures write the live camera, which projects itself imperatively once per frame. The only reactive
 * camera value published to the scene tree is the zoom, because zoom changes screen-space chrome metrics;
 * position never re-renders a component.
 *
 * Two extents with two jobs: `size` is the host, which is what fitting, centring and wheel normalization
 * mean by the viewport; `surface` is what is painted and culled, and it is bounded by the window so the
 * host's edges can move without the stage reacting at all.
 *
 * The camera is held in the host's own coordinate space and is never compensated for host movement. When
 * a sidebar moves the work area, the scene moves with it through layout alone, without a canvas draw.
 * Compensation would have to read the host's new origin, which cannot be known earlier than the layout
 * that created it, so it always lands one frame after the pixels it is correcting - and a one-frame
 * correction of the whole scene is exactly the jump it was meant to prevent.
 */
export function useStageViewport(options: StageViewportOptions) {
  const measuredSize = useElementSize(options.host, { width: 1, height: 1 });
  const size = reactive({
    width: computed(() => Math.max(1, measuredSize.width.value)),
    height: computed(() => Math.max(1, measuredSize.height.value)),
  });
  const surface = useStageSurfaceSize(reactive(useWindowSize({ includeScrollbar: false })));
  const camera = createCanvasCamera({ ...options.initialViewport });
  const zoom = shallowRef(camera.get().zoom);
  const animated = shallowRef(camera.get());
  const { key, spacePressed } = useViewportSpaceKey();
  let animationGeneration = 0;
  const pan = createCameraPan({
    abortAnimation: () => animationGeneration++,
    camera,
    navigationEnabled: options.navigationEnabled,
    spacePressed,
  });

  camera.observeFrames((value) => {
    zoom.value = value.zoom;
    options.project(value);
  });
  camera.observeSettled(options.commit);
  watch(animated, (value) => camera.set(value));
  onScopeDispose(camera.dispose);

  function takeCameraControl() {
    animationGeneration++;
    pan.release();
  }

  function flushViewport() {
    takeCameraControl();
    camera.settle();
  }

  function setViewport(next: CanvasViewport) {
    takeCameraControl();
    camera.set({ ...next });
  }

  function animateViewport(next: CanvasViewport) {
    takeCameraControl();
    const generation = animationGeneration;
    animated.value = camera.get();
    void transition(
      animated,
      { ...animated.value },
      { ...next },
      {
        duration: ANIMATION_MS,
        easing: TransitionPresets.easeOutCubic,
        abort: () => generation !== animationGeneration,
        interpolation: (from, to, progress) => ({
          x: from.x + (to.x - from.x) * progress,
          y: from.y + (to.y - from.y) * progress,
          zoom: from.zoom + (to.zoom - from.zoom) * progress,
        }),
      },
    );
  }

  function wheel(event: Konva.KonvaEventObject<WheelEvent>) {
    if (isScrollableTarget(event.evt.target)) return;
    event.evt.preventDefault();
    if (!options.navigationEnabled()) return;
    const pointer = event.target.getStage()?.getPointerPosition();
    if (!pointer) return;
    const current = camera.get();
    const delta = normalizedWheelDelta(event.evt, size.height);
    takeCameraControl();
    if (event.evt.ctrlKey || event.evt.metaKey) {
      camera.set(zoomViewportAt(current, pointer, current.zoom * Math.exp(-delta.y * 0.002)));
      return;
    }
    camera.set({ x: current.x - delta.x, y: current.y - delta.y, zoom: current.zoom });
  }

  watch(options.navigationEnabled, (enabled) => {
    if (!enabled) flushViewport();
  });
  onBeforeUnmount(flushViewport);

  /** Releasing the hold lets settlement run on the next quiet frame, keeping it out of the pointerup frame. */
  function endPan(event?: PointerEvent) {
    if (pan.owns(event)) pan.release();
  }

  return {
    animateViewport,
    beginPan: (event: Konva.KonvaEventObject<PointerEvent>, handTool = false) => pan.begin(event, handTool),
    camera,
    endPan,
    flushViewport,
    getViewport: camera.get,
    key,
    movePan: pan.move,
    panning: pan.panning,
    setViewport,
    size,
    surface,
    spacePressed,
    wheel,
    zoom,
  };
}

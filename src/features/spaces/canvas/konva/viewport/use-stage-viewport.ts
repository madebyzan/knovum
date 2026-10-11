import { TransitionPresets, transition, useElementSize } from "@vueuse/core";
import type Konva from "konva";
import { computed, onBeforeUnmount, reactive, shallowRef, watch, type Ref } from "vue";

import { isScrollableTarget } from "../../../../../foundation/dom/scrollable-target";
import type { CanvasPoint, CanvasViewport } from "../../contracts/canvas-editor-port";
import { useViewportMotion } from "./use-viewport-motion";
import { useViewportSpaceKey } from "./use-viewport-space-key";
import { normalizedWheelDelta, zoomViewportAt } from "./viewport-math";

interface StageViewportOptions {
  host: Readonly<Ref<HTMLElement | null>>;
  initialViewport: CanvasViewport;
  isNavigationEnabled(): boolean;
  update(viewport: CanvasViewport): void;
}

export function useStageViewport(options: StageViewportOptions) {
  const stageSizeStep = 128;
  const measuredSize = useElementSize(options.host, { width: 1, height: 1 });
  const size = reactive({
    width: computed(() => Math.max(1, measuredSize.width.value)),
    height: computed(() => Math.max(1, measuredSize.height.value)),
  });
  const stageSize = reactive({
    width: computed(() => Math.ceil(size.width / stageSizeStep) * stageSizeStep),
    height: computed(() => Math.ceil(size.height / stageSizeStep) * stageSizeStep),
  });
  const viewport = shallowRef<CanvasViewport>({ ...options.initialViewport });
  let publishedViewport = viewport.value;
  let animationGeneration = 0;
  const panStart = shallowRef<{ pointerId: number; pointer: CanvasPoint; viewport: CanvasViewport }>();
  const panning = computed(() => panStart.value !== undefined);
  const { moving, markMoving } = useViewportMotion();
  watch(viewport, markMoving);
  const { key, spacePressed } = useViewportSpaceKey();
  let panFrame: number | undefined;
  let pendingPanViewport: CanvasViewport | undefined;
  let publicationFrame: number | undefined;

  function schedulePublication() {
    if (publicationFrame !== undefined) return;
    publicationFrame = requestAnimationFrame(() => {
      publicationFrame = undefined;
      publishViewport();
    });
  }

  function publishViewport() {
    if (viewport.value === publishedViewport) return;
    publishedViewport = viewport.value;
    options.update(viewport.value);
  }

  function takeViewportControl() {
    animationGeneration++;
    if (panFrame !== undefined) cancelAnimationFrame(panFrame);
    panFrame = undefined;
    if (pendingPanViewport) viewport.value = pendingPanViewport;
    pendingPanViewport = undefined;
    panStart.value = undefined;
    if (publicationFrame !== undefined) cancelAnimationFrame(publicationFrame);
    publicationFrame = undefined;
  }

  function flushViewport() {
    takeViewportControl();
    publishViewport();
  }

  function getViewport() {
    return pendingPanViewport ?? viewport.value;
  }

  function setViewport(next: CanvasViewport, notify = true) {
    takeViewportControl();
    viewport.value = { ...next };
    if (notify) schedulePublication();
    else publishedViewport = viewport.value;
  }

  function animateViewport(next: CanvasViewport) {
    takeViewportControl();
    const generation = animationGeneration;
    void transition(
      viewport,
      { ...viewport.value },
      { ...next },
      {
        duration: 180,
        easing: TransitionPresets.easeOutCubic,
        abort: () => generation !== animationGeneration,
        interpolation: (from, to, progress) => ({
          x: from.x + (to.x - from.x) * progress,
          y: from.y + (to.y - from.y) * progress,
          zoom: from.zoom + (to.zoom - from.zoom) * progress,
        }),
      },
    ).then(() => {
      if (generation === animationGeneration) publishViewport();
      return undefined;
    });
  }

  function wheel(event: Konva.KonvaEventObject<WheelEvent>) {
    if (isScrollableTarget(event.evt.target)) return;
    event.evt.preventDefault();
    if (!options.isNavigationEnabled()) return;
    const pointer = event.target.getStage()?.getPointerPosition();
    if (!pointer) return;
    const current = getViewport();
    const delta = normalizedWheelDelta(event.evt, size.height);
    if (event.evt.ctrlKey || event.evt.metaKey) {
      setViewport(zoomViewportAt(current, pointer, current.zoom * Math.exp(-delta.y * 0.002)));
      return;
    }
    setViewport({
      x: current.x - delta.x,
      y: current.y - delta.y,
      zoom: current.zoom,
    });
  }

  function beginPan(event: Konva.KonvaEventObject<PointerEvent>, handTool = false) {
    if (!options.isNavigationEnabled()) return false;
    if (event.evt.button !== 1 && !(event.evt.button === 0 && (handTool || spacePressed.value))) return false;
    if (panStart.value) return true;
    const pointer = event.target.getStage()?.getPointerPosition();
    if (!pointer) return false;
    event.evt.preventDefault();
    takeViewportControl();
    markMoving();
    panStart.value = { pointerId: event.evt.pointerId, pointer, viewport: viewport.value };
    return true;
  }

  function movePan(event: Konva.KonvaEventObject<PointerEvent>) {
    const start = panStart.value;
    if (!start || event.evt.pointerId !== start.pointerId) return;
    const pointer = event.target.getStage()?.getPointerPosition();
    if (!pointer) return;
    pendingPanViewport = {
      x: start.viewport.x + pointer.x - start.pointer.x,
      y: start.viewport.y + pointer.y - start.pointer.y,
      zoom: start.viewport.zoom,
    };
    if (panFrame !== undefined) return;
    panFrame = requestAnimationFrame(flushPanViewport);
  }

  function endPan(event?: PointerEvent) {
    if (!panStart.value || (event && event.pointerId !== panStart.value.pointerId)) return;
    flushViewport();
  }

  function flushPanViewport() {
    panFrame = undefined;
    if (!pendingPanViewport) return;
    viewport.value = pendingPanViewport;
    pendingPanViewport = undefined;
    publishViewport();
  }

  watch(options.isNavigationEnabled, (enabled) => {
    if (!enabled) flushViewport();
  });
  onBeforeUnmount(flushViewport);

  return {
    beginPan,
    animateViewport,
    endPan,
    flushViewport,
    getViewport,
    key,
    movePan,
    moving,
    panning,
    setViewport,
    size,
    stageSize,
    spacePressed,
    viewport,
    wheel,
  };
}

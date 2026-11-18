import { useEventListener } from "@vueuse/core";
import { computed, onScopeDispose, shallowRef, type Ref } from "vue";

import type { CanvasEditorPort, CanvasViewport } from "../../contracts/canvas-editor-port";
import {
  isPointInMinimapRect,
  minimapPointToWorld,
  viewportCenteredAtWorld,
  type MinimapTransform,
  type MinimapViewportProjection,
} from "../../minimap/minimap-transform";
import { normalizedWheelDelta, zoomViewportAt } from "../../viewport-math";

interface MinimapNavigationOptions {
  readonly surface: Readonly<Ref<HTMLElement | null>>;
  readonly editor: () => CanvasEditorPort | undefined;
  readonly viewport: () => CanvasViewport | undefined;
  readonly hostSize: () => { readonly width: number; readonly height: number };
  readonly transform: () => MinimapTransform | undefined;
  readonly viewportProjection: () => MinimapViewportProjection | undefined;
}

type DragSession =
  | {
      readonly mode: "viewport-drag";
      readonly pointerId: number;
      readonly startX: number;
      readonly startY: number;
      readonly startViewport: CanvasViewport;
      readonly scale: number;
    }
  | {
      readonly mode: "scrub";
      readonly pointerId: number;
      readonly startX: number;
      readonly startY: number;
      readonly moved: boolean;
    };

const DRAG_THRESHOLD_PX = 3;

/** Owns direct-manipulation panning, click-to-jump, wheel navigation, and double-click fit on the mini map. */
export function useMinimapNavigation(options: MinimapNavigationOptions) {
  const session = shallowRef<DragSession>();
  const dragging = computed(() => session.value !== undefined);

  function releaseCapture(pointerId: number) {
    const element = options.surface.value;
    if (element?.hasPointerCapture?.(pointerId)) {
      element.releasePointerCapture(pointerId);
    }
  }

  function endSession() {
    if (!session.value) return;
    releaseCapture(session.value.pointerId);
    session.value = undefined;
  }

  function localPoint(clientX: number, clientY: number) {
    const rect = options.surface.value?.getBoundingClientRect();
    if (!rect) return undefined;
    return { x: clientX - rect.left, y: clientY - rect.top };
  }

  function centerAtClient(clientX: number, clientY: number, animate: boolean) {
    const editor = options.editor();
    const viewport = options.viewport();
    const transform = options.transform();
    const point = localPoint(clientX, clientY);
    if (!editor || !viewport || !transform || !point) return;
    const next = viewportCenteredAtWorld(
      minimapPointToWorld(point, transform),
      viewport.zoom,
      options.hostSize(),
    );
    if (animate) editor.navigateViewport(next);
    else editor.setViewport(next);
  }

  function onPointerDown(event: PointerEvent) {
    if (event.button !== 0) return;
    const projection = options.viewportProjection();
    const transform = options.transform();
    const viewport = options.viewport();
    const point = localPoint(event.clientX, event.clientY);
    if (!projection || !transform || !viewport || !point) return;
    event.preventDefault();
    options.surface.value?.setPointerCapture?.(event.pointerId);
    if (projection.intersectsMap && isPointInMinimapRect(point, projection.rect)) {
      session.value = {
        mode: "viewport-drag",
        pointerId: event.pointerId,
        startX: event.clientX,
        startY: event.clientY,
        startViewport: viewport,
        scale: transform.scale,
      };
      return;
    }
    session.value = {
      mode: "scrub",
      pointerId: event.pointerId,
      startX: event.clientX,
      startY: event.clientY,
      moved: false,
    };
    centerAtClient(event.clientX, event.clientY, true);
  }

  function onPointerMove(event: PointerEvent) {
    const active = session.value;
    if (!active || active.pointerId !== event.pointerId) return;
    if (active.mode === "viewport-drag") {
      const editor = options.editor();
      if (!editor) return;
      const worldDx = (event.clientX - active.startX) / active.scale;
      const worldDy = (event.clientY - active.startY) / active.scale;
      editor.setViewport({
        x: active.startViewport.x - worldDx * active.startViewport.zoom,
        y: active.startViewport.y - worldDy * active.startViewport.zoom,
        zoom: active.startViewport.zoom,
      });
      return;
    }
    const moved =
      active.moved ||
      Math.hypot(event.clientX - active.startX, event.clientY - active.startY) > DRAG_THRESHOLD_PX;
    if (!moved) return;
    if (!active.moved) session.value = { ...active, moved: true };
    centerAtClient(event.clientX, event.clientY, false);
  }

  function onPointerUp(event: PointerEvent) {
    if (session.value?.pointerId === event.pointerId) endSession();
  }

  function onWheel(event: WheelEvent) {
    event.preventDefault();
    if (session.value) return;
    const editor = options.editor();
    const viewport = options.viewport();
    if (!editor || !viewport) return;
    const hostSize = options.hostSize();
    const delta = normalizedWheelDelta(event, hostSize.height);
    if (event.ctrlKey || event.metaKey) {
      const center = { x: hostSize.width / 2, y: hostSize.height / 2 };
      editor.setViewport(zoomViewportAt(viewport, center, viewport.zoom * Math.exp(-delta.y * 0.002)));
      return;
    }
    editor.setViewport({ x: viewport.x - delta.x, y: viewport.y - delta.y, zoom: viewport.zoom });
  }

  function onDoubleClick() {
    options.editor()?.zoomToFit();
  }

  useEventListener(globalThis, "keydown", (event: KeyboardEvent) => {
    if (event.key === "Escape" && session.value) endSession();
  });
  useEventListener(globalThis, "blur", endSession);
  onScopeDispose(endSession);

  return {
    cancel: endSession,
    dragging,
    onDoubleClick,
    onPointerCancel: onPointerUp,
    onPointerDown,
    onPointerMove,
    onPointerUp,
    onWheel,
  };
}

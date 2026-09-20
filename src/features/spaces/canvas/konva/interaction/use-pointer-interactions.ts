import { useEventListener } from "@vueuse/core";
import type Konva from "konva";
import { onScopeDispose, shallowRef, watch, type Ref } from "vue";

import type { CanvasPoint } from "../../contract/canvas-editor";
import type { CanvasTool } from "../../contract/canvas-tool";
import type { useCanvasSelection } from "../selection/use-canvas-selection";
import type { useStageViewport } from "../viewport/use-stage-viewport";

interface PointerInteractionOptions {
  cancelActiveGesture(): boolean;
  resolveHit(id: string): string;
  sectionDrawing: {
    begin(point: CanvasPoint): void;
    move(point: CanvasPoint): void;
    finish(): Promise<void>;
    cancel(): void;
  };
  host: Readonly<Ref<HTMLElement | null>>;
  selection: ReturnType<typeof useCanvasSelection>;
  viewport: ReturnType<typeof useStageViewport>;
  tool: Readonly<Ref<CanvasTool>>;
  createTextNode(kind: "note" | "text", point: CanvasPoint): Promise<void>;
  reportError(error: unknown): void;
}

export function usePointerInteractions(options: PointerInteractionOptions) {
  const lastCanvasPoint = shallowRef<CanvasPoint>();
  let capturedPointer: { element: Element; id: number } | undefined;
  let pressedSelection: { id: number; nodeId: string; origin: CanvasPoint; moved: boolean } | undefined;
  let creating = false;
  watch(
    options.viewport.panning,
    (active) => {
      if (!active) releaseCapture();
    },
    { flush: "sync" },
  );
  useEventListener(globalThis, "blur", () => pointerCancel());
  onScopeDispose(releaseCapture);
  const updatePoint = (event: Konva.KonvaEventObject<PointerEvent>) => {
    const point = event.target.getStage()?.getRelativePointerPosition();
    if (point) lastCanvasPoint.value = point;
  };
  function pointerDown(event: Konva.KonvaEventObject<PointerEvent>) {
    if (capturedPointer && capturedPointer.id !== event.evt.pointerId) return;
    updatePoint(event);
    options.host.value?.focus();
    capture(event.evt);
    if (!beginToolInteraction(event)) options.selection.beginMarquee(event);
  }
  function pointerMove(event: Konva.KonvaEventObject<PointerEvent>) {
    if (capturedPointer && capturedPointer.id !== event.evt.pointerId) return;
    updatePoint(event);
    if (pressedSelection?.id === event.evt.pointerId) {
      const distance = Math.hypot(
        event.evt.clientX - pressedSelection.origin.x,
        event.evt.clientY - pressedSelection.origin.y,
      );
      if (distance >= 3) pressedSelection.moved = true;
    }
    if (options.tool.value === "section" && lastCanvasPoint.value && !options.viewport.panning.value)
      options.sectionDrawing.move(lastCanvasPoint.value);
    else if (options.viewport.panning.value) options.viewport.movePan(event);
    else options.selection.moveMarquee(event);
  }
  function pointerUp(event?: PointerEvent) {
    if (event && capturedPointer && capturedPointer.id !== event.pointerId) return;
    if (options.viewport.panning.value) options.viewport.endPan(event);
    else if (options.tool.value === "section") void options.sectionDrawing.finish();
    else options.selection.endMarquee();
    if (pressedSelection && (!event || pressedSelection.id === event.pointerId)) {
      if (!pressedSelection.moved) options.selection.setNodeIds([pressedSelection.nodeId]);
      pressedSelection = undefined;
    }
    releaseCapture();
  }
  function pointerCancel(event?: PointerEvent) {
    if (event && capturedPointer && capturedPointer.id !== event.pointerId) return;
    options.cancelActiveGesture();
    options.sectionDrawing.cancel();
    options.viewport.endPan();
    options.selection.cancelMarquee();
    pressedSelection = undefined;
    releaseCapture();
  }
  function nodePointerDown(nodeId: string, event: Konva.KonvaEventObject<PointerEvent>) {
    event.cancelBubble = true;
    if (capturedPointer && capturedPointer.id !== event.evt.pointerId) return;
    updatePoint(event);
    options.host.value?.focus();
    capture(event.evt);
    if (beginToolInteraction(event)) return;
    nodeId = options.resolveHit(nodeId);
    const additive = event.evt.metaKey || event.evt.ctrlKey || event.evt.shiftKey;
    pressedSelection =
      !additive && options.selection.nodeIds.value.has(nodeId)
        ? {
            id: event.evt.pointerId,
            nodeId,
            origin: { x: event.evt.clientX, y: event.evt.clientY },
            moved: false,
          }
        : undefined;
    options.selection.selectNode(nodeId, event.evt);
  }
  function beginToolInteraction(event: Konva.KonvaEventObject<PointerEvent>) {
    if (options.viewport.beginPan(event, options.tool.value === "pan")) return true;
    const tool = options.tool.value;
    if (event.evt.button !== 0 || tool === "select" || tool === "pan") return false;
    const point = lastCanvasPoint.value;
    if (tool === "section") {
      if (point) options.sectionDrawing.begin(point);
      return true;
    }
    if (point && !creating) {
      creating = true;
      void options
        .createTextNode(tool, point)
        .catch(options.reportError)
        .finally(() => {
          creating = false;
        });
    }
    return true;
  }
  function capture(event: PointerEvent) {
    const element = event.currentTarget;
    if (
      !(element instanceof Element) ||
      !element.hasPointerCapture ||
      element.hasPointerCapture(event.pointerId)
    ) {
      return;
    }
    element.setPointerCapture(event.pointerId);
    capturedPointer = { element, id: event.pointerId };
  }
  function releaseCapture() {
    if (!capturedPointer) return;
    const { element, id } = capturedPointer;
    capturedPointer = undefined;
    if (element.hasPointerCapture(id)) element.releasePointerCapture(id);
  }
  return { lastCanvasPoint, nodePointerDown, pointerCancel, pointerDown, pointerMove, pointerUp };
}

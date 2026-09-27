import { useEventListener } from "@vueuse/core";
import type Konva from "konva";
import { onScopeDispose, shallowRef, watch, type Ref } from "vue";

import { SECTION_DEFAULT_SIZE, SECTION_MINIMUM_SIZE } from "../../../model/structure/section-metrics";
import type { CanvasPoint, CanvasRect } from "../../contracts/canvas-editor-port";
import type { CanvasTool } from "../../contracts/canvas-tool";
import type { useCanvasSelection } from "../selection/use-canvas-selection";
import type { useStageViewport } from "../viewport/use-stage-viewport";

interface PointerInteractionOptions {
  cancelActiveGesture(): boolean;
  host: Readonly<Ref<HTMLElement | null>>;
  selection: ReturnType<typeof useCanvasSelection>;
  viewport: ReturnType<typeof useStageViewport>;
  tool: Readonly<Ref<CanvasTool>>;
  createTextNode(kind: "note" | "text", point: CanvasPoint): Promise<void>;
  createSection(frame: CanvasRect): Promise<void>;
  reportError(error: unknown): void;
}

export function usePointerInteractions(options: PointerInteractionOptions) {
  const lastCanvasPoint = shallowRef<CanvasPoint>();
  const sectionDraft = shallowRef<CanvasRect>();
  let sectionOrigin: CanvasPoint | undefined;
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
    if (options.viewport.panning.value) options.viewport.movePan(event);
    else if (sectionOrigin && lastCanvasPoint.value)
      sectionDraft.value = sectionFrame(sectionOrigin, lastCanvasPoint.value);
    else options.selection.moveMarquee(event);
  }
  function pointerUp(event?: PointerEvent) {
    if (event && capturedPointer && capturedPointer.id !== event.pointerId) return;
    if (options.viewport.panning.value) options.viewport.endPan(event);
    else if (sectionOrigin) finishSection();
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
    options.viewport.endPan();
    options.selection.cancelMarquee();
    sectionOrigin = undefined;
    sectionDraft.value = undefined;
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
    if (tool === "section" && point) beginSection(point);
    else if (point && !creating && isTextCreationTool(tool)) createTextNode(tool, point);
    return true;
  }
  function beginSection(point: CanvasPoint) {
    sectionOrigin = { ...point };
    sectionDraft.value = { ...point, ...SECTION_DEFAULT_SIZE };
  }
  function createTextNode(tool: "note" | "text", point: CanvasPoint) {
    creating = true;
    void options
      .createTextNode(tool, point)
      .catch(options.reportError)
      .finally(() => {
        creating = false;
      });
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
  function finishSection() {
    const frame = sectionDraft.value;
    sectionOrigin = undefined;
    sectionDraft.value = undefined;
    if (!frame || creating) return;
    creating = true;
    void options
      .createSection(frame)
      .catch(options.reportError)
      .finally(() => {
        creating = false;
      });
  }
  return {
    lastCanvasPoint,
    nodePointerDown,
    pointerCancel,
    pointerDown,
    pointerMove,
    pointerUp,
    sectionDraft,
  };
}

function isTextCreationTool(tool: CanvasTool): tool is "note" | "text" {
  return tool === "note" || tool === "text";
}

function sectionFrame(origin: CanvasPoint, point: CanvasPoint): CanvasRect {
  const dx = point.x - origin.x;
  const dy = point.y - origin.y;
  if (Math.hypot(dx, dy) < 4) return { ...origin, ...SECTION_DEFAULT_SIZE };
  const width = Math.max(Math.abs(dx), SECTION_MINIMUM_SIZE.width);
  const height = Math.max(Math.abs(dy), SECTION_MINIMUM_SIZE.height);
  return {
    x: dx < 0 ? origin.x - width : origin.x,
    y: dy < 0 ? origin.y - height : origin.y,
    width,
    height,
  };
}

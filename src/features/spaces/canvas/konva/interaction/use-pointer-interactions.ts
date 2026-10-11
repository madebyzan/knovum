import { useEventListener } from "@vueuse/core";
import type Konva from "konva";
import { onScopeDispose, shallowRef, watch, type Ref } from "vue";

import { SECTION_DEFAULT_SIZE } from "../../../model/structure/section-metrics";
import { sectionDrawFrame } from "../../../structure/section-draw-candidates";
import type { CanvasPoint, CanvasRect } from "../../contracts/canvas-editor-port";
import type { CanvasTool } from "../../contracts/canvas-tool";
import type { useCanvasSelection } from "../selection/use-canvas-selection";
import type { useStageViewport } from "../viewport/use-stage-viewport";
import { createCanvasPointerCapture } from "./canvas-pointer-capture";

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
  const pressedCanvasPoint = shallowRef<CanvasPoint>();
  const sectionDraft = shallowRef<CanvasRect>();
  let sectionOrigin: CanvasPoint | undefined;
  const pointerCapture = createCanvasPointerCapture();
  let pressedSelection: { id: number; selectOnly(): void; origin: CanvasPoint; moved: boolean } | undefined;
  let creating = false;
  watch(
    options.viewport.panning,
    (active) => {
      if (!active) pointerCapture.release();
    },
    { flush: "sync" },
  );
  useEventListener(globalThis, "blur", () => pointerCancel());
  onScopeDispose(pointerCapture.release);
  const updatePoint = (event: Konva.KonvaEventObject<PointerEvent>) => {
    const point = event.target.getStage()?.getRelativePointerPosition();
    if (point) lastCanvasPoint.value = point;
  };
  function pointerDown(event: Konva.KonvaEventObject<PointerEvent>) {
    if (!pointerCapture.accepts(event.evt.pointerId)) return;
    updatePoint(event);
    pressedCanvasPoint.value = lastCanvasPoint.value;
    options.host.value?.focus();
    pointerCapture.capture(event.evt);
    if (!beginToolInteraction(event)) options.selection.beginMarquee(event);
  }
  function pointerMove(event: Konva.KonvaEventObject<PointerEvent>) {
    if (!pointerCapture.accepts(event.evt.pointerId)) return;
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
      sectionDraft.value = sectionDrawFrame(sectionOrigin, lastCanvasPoint.value);
    else options.selection.moveMarquee(event);
  }
  function pointerUp(event?: PointerEvent) {
    if (event && !pointerCapture.accepts(event.pointerId)) return;
    if (options.viewport.panning.value) options.viewport.endPan(event);
    else if (sectionOrigin) finishSection();
    else options.selection.endMarquee();
    if (pressedSelection && (!event || pressedSelection.id === event.pointerId)) {
      if (!pressedSelection.moved) pressedSelection.selectOnly();
      pressedSelection = undefined;
    }
    pressedCanvasPoint.value = undefined;
    pointerCapture.release();
  }
  function pointerCancel(event?: PointerEvent) {
    if (event && !pointerCapture.accepts(event.pointerId)) return;
    options.cancelActiveGesture();
    options.viewport.endPan();
    options.selection.cancelMarquee();
    sectionOrigin = undefined;
    sectionDraft.value = undefined;
    pressedSelection = undefined;
    pressedCanvasPoint.value = undefined;
    pointerCapture.release();
  }
  function nodePointerDown(nodeId: string, event: Konva.KonvaEventObject<PointerEvent>) {
    const structureId = options.selection.structureSelectionForNode(nodeId);
    if (structureId) return structurePointerDown(structureId, event);
    targetPointerDown(
      event,
      options.selection.nodeIds.value.has(nodeId),
      () => options.selection.selectNode(nodeId, event.evt),
      () => options.selection.setNodeIds([nodeId]),
    );
  }
  function structurePointerDown(structureNodeId: string, event: Konva.KonvaEventObject<PointerEvent>) {
    targetPointerDown(
      event,
      options.selection.structureNodeIds.value.has(structureNodeId),
      () => options.selection.selectStructureNode(structureNodeId, event.evt),
      () => options.selection.setStructureNodeIds([structureNodeId]),
    );
  }
  function targetPointerDown(
    event: Konva.KonvaEventObject<PointerEvent>,
    selected: boolean,
    select: () => void,
    selectOnly: () => void,
  ) {
    event.cancelBubble = true;
    if (!pointerCapture.accepts(event.evt.pointerId)) return;
    updatePoint(event);
    pressedCanvasPoint.value = lastCanvasPoint.value;
    options.host.value?.focus();
    pointerCapture.capture(event.evt);
    if (beginToolInteraction(event)) return;
    const additive = event.evt.metaKey || event.evt.ctrlKey || event.evt.shiftKey;
    pressedSelection =
      !additive && selected && event.evt.button === 0
        ? {
            id: event.evt.pointerId,
            selectOnly,
            origin: { x: event.evt.clientX, y: event.evt.clientY },
            moved: false,
          }
        : undefined;
    select();
  }
  function beginToolInteraction(event: Konva.KonvaEventObject<PointerEvent>) {
    if (options.viewport.beginPan(event, options.tool.value === "pan")) return true;
    const tool = options.tool.value;
    if (tool === "pan" || options.viewport.spacePressed.value) return true;
    if (event.evt.button !== 0 || tool === "select") return false;
    const point = lastCanvasPoint.value;
    if (!point || creating) return true;
    if (tool === "section") beginSection(point);
    else runCreation(() => options.createTextNode(tool, point));
    return true;
  }
  function beginSection(point: CanvasPoint) {
    sectionOrigin = { ...point };
    sectionDraft.value = { ...point, ...SECTION_DEFAULT_SIZE };
  }
  function runCreation(operation: () => Promise<void>) {
    creating = true;
    void operation()
      .catch(options.reportError)
      .finally(() => {
        creating = false;
      });
  }
  function finishSection() {
    const frame = sectionDraft.value;
    sectionOrigin = undefined;
    sectionDraft.value = undefined;
    if (!frame || creating) return;
    runCreation(() => options.createSection(frame));
  }
  return {
    lastCanvasPoint,
    pressedCanvasPoint,
    nodePointerDown,
    pointerCancel,
    pointerDown,
    pointerMove,
    pointerUp,
    sectionDraft,
    structurePointerDown,
    markSelectionDragged: () => {
      if (pressedSelection) pressedSelection.moved = true;
    },
  };
}

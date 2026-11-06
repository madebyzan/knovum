import type Konva from "konva";
import { computed, shallowRef, type Ref } from "vue";

import type { CanvasRect } from "../../contracts/canvas-editor-port";
import type { CanvasNodeLayoutV1 } from "../../schema/document";
import { selectionRectIntersects } from "./selection-geometry";

interface CanvasMarqueeSelectionOptions {
  clear(): void;
  layouts: Readonly<Ref<readonly CanvasNodeLayoutV1[]>>;
  nodeIds: Ref<Set<string>>;
  sectionBounds: Readonly<Ref<ReadonlyMap<string, CanvasRect>>>;
  notify(): void;
  structureNodeIds: Ref<Set<string>>;
  structureSelectionForNode(nodeId: string): string | undefined;
}

export function useCanvasMarqueeSelection(options: CanvasMarqueeSelectionOptions) {
  const marquee = shallowRef<{ start: { x: number; y: number }; end: { x: number; y: number } }>();
  const marqueeRect = computed(() => {
    const value = marquee.value;
    if (!value) return undefined;
    return {
      x: Math.min(value.start.x, value.end.x),
      y: Math.min(value.start.y, value.end.y),
      width: Math.abs(value.end.x - value.start.x),
      height: Math.abs(value.end.y - value.start.y),
    };
  });

  function begin(event: Konva.KonvaEventObject<PointerEvent>) {
    if (event.evt.button !== 0 || event.target !== event.target.getStage()) return false;
    const point = event.target.getStage()?.getRelativePointerPosition();
    if (!point) return false;
    marquee.value = { start: point, end: point };
    if (!(event.evt.metaKey || event.evt.ctrlKey || event.evt.shiftKey)) options.clear();
    return true;
  }

  function move(event: Konva.KonvaEventObject<PointerEvent>) {
    if (!marquee.value) return;
    const point = event.target.getStage()?.getRelativePointerPosition();
    if (point) marquee.value = { ...marquee.value, end: point };
  }

  function end() {
    const bounds = marqueeRect.value;
    marquee.value = undefined;
    if (!bounds) return;
    const nextNodes = new Set(options.nodeIds.value);
    const nextStructureNodeIds = new Set(options.structureNodeIds.value);
    // A fully enclosed section is selected as a container. Intersecting only its contents still
    // selects those contents, so framing inside a section does not unexpectedly move the section.
    for (const [id, frame] of options.sectionBounds.value) {
      if (contains(bounds, frame)) nextStructureNodeIds.add(id);
    }
    for (const { nodeId, frame } of options.layouts.value) {
      if (!selectionRectIntersects(frame, bounds)) continue;
      const structureNodeId = options.structureSelectionForNode(nodeId);
      if (structureNodeId) nextStructureNodeIds.add(structureNodeId);
      else nextNodes.add(nodeId);
    }
    options.nodeIds.value = nextNodes;
    options.structureNodeIds.value = nextStructureNodeIds;
    options.notify();
  }

  function cancel() {
    marquee.value = undefined;
  }

  return { begin, cancel, end, marqueeRect, move };
}

function contains(outer: CanvasRect, inner: CanvasRect) {
  return (
    inner.x >= outer.x &&
    inner.y >= outer.y &&
    inner.x + inner.width <= outer.x + outer.width &&
    inner.y + inner.height <= outer.y + outer.height
  );
}

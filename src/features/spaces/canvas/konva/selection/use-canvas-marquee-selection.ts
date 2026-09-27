import type Konva from "konva";
import { computed, shallowRef, type Ref } from "vue";

import type { CanvasNodeLayoutV1 } from "../../schema/document";
import { selectionRectIntersects } from "./selection-geometry";

export function useCanvasMarqueeSelection(options: {
  clear(): void;
  layouts: Readonly<Ref<readonly CanvasNodeLayoutV1[]>>;
  nodeIds: Ref<Set<string>>;
  notify(): void;
  structureNodeIds: Ref<Set<string>>;
  structureSelectionForNode(nodeId: string): string | undefined;
}) {
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

import type Konva from "konva";
import { computed, shallowRef, watch, type Ref } from "vue";

import type { CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import type { CanvasSelectionSnapshot } from "../../contract/canvas-editor";
import { createCanvasSelectionSnapshot } from "../../document/canvas-selection-snapshot";

interface CanvasSelectionOptions {
  layouts: Readonly<Ref<readonly CanvasNodeLayoutV1[]>>;
  relationBounds: Readonly<Ref<ReadonlyMap<string, { x: number; y: number; width: number; height: number }>>>;
  changed(selection: CanvasSelectionSnapshot): void;
}

export function useCanvasSelection(options: CanvasSelectionOptions) {
  const nodeIds = shallowRef(new Set<string>());
  const relationIds = shallowRef(new Set<string>());
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

  function selectNode(nodeId: string, event: PointerEvent) {
    const additive = event.metaKey || event.ctrlKey || event.shiftKey;
    if (!additive && nodeIds.value.has(nodeId)) return;
    const next = new Set(additive ? nodeIds.value : []);
    if (next.has(nodeId) && additive) next.delete(nodeId);
    else next.add(nodeId);
    nodeIds.value = next;
    relationIds.value = new Set();
    notify();
  }

  function selectRelation(id: string, event: PointerEvent) {
    const additive = event.metaKey || event.ctrlKey || event.shiftKey;
    const next = new Set(additive ? relationIds.value : []);
    if (next.has(id) && additive) next.delete(id);
    else next.add(id);
    nodeIds.value = new Set();
    relationIds.value = next;
    notify();
  }

  function beginMarquee(event: Konva.KonvaEventObject<PointerEvent>) {
    if (event.evt.button !== 0 || event.target !== event.target.getStage()) return false;
    const point = event.target.getStage()?.getRelativePointerPosition();
    if (!point) return false;
    marquee.value = { start: point, end: point };
    if (!(event.evt.metaKey || event.evt.ctrlKey || event.evt.shiftKey)) clear();
    return true;
  }

  function moveMarquee(event: Konva.KonvaEventObject<PointerEvent>) {
    if (!marquee.value) return;
    const point = event.target.getStage()?.getRelativePointerPosition();
    if (point) marquee.value = { ...marquee.value, end: point };
  }

  function endMarquee() {
    const bounds = marqueeRect.value;
    marquee.value = undefined;
    if (!bounds) return;
    nodeIds.value = new Set([
      ...nodeIds.value,
      ...options.layouts.value.filter(({ frame }) => intersects(frame, bounds)).map(({ nodeId }) => nodeId),
    ]);
    notify();
  }

  function cancelMarquee() {
    marquee.value = undefined;
  }

  function setNodeIds(ids: readonly string[]) {
    nodeIds.value = new Set(ids);
    relationIds.value = new Set();
    notify();
  }

  function clear() {
    nodeIds.value = new Set();
    relationIds.value = new Set();
    notify();
  }

  function notify() {
    options.changed(
      createCanvasSelectionSnapshot(
        options.layouts.value,
        nodeIds.value,
        relationIds.value,
        options.relationBounds.value,
      ),
    );
  }

  watch(options.layouts, (currentLayouts) => {
    if (!nodeIds.value.size) return;
    const visibleIds = new Set(currentLayouts.map(({ nodeId }) => nodeId));
    const retained = new Set([...nodeIds.value].filter((id) => visibleIds.has(id)));
    if (retained.size !== nodeIds.value.size) nodeIds.value = retained;
    notify();
  });
  watch(options.relationBounds, (currentBounds) => {
    if (!relationIds.value.size) return;
    const retained = new Set([...relationIds.value].filter((id) => currentBounds.has(id)));
    if (retained.size === relationIds.value.size) return;
    relationIds.value = retained;
    notify();
  });

  return {
    beginMarquee,
    cancelMarquee,
    clear,
    endMarquee,
    marqueeRect,
    moveMarquee,
    nodeIds,
    relationIds,
    selectNode,
    selectRelation,
    setNodeIds,
  };
}

function intersects(
  left: { x: number; y: number; width: number; height: number },
  right: { x: number; y: number; width: number; height: number },
) {
  return (
    left.x <= right.x + right.width &&
    left.x + left.width >= right.x &&
    left.y <= right.y + right.height &&
    left.y + left.height >= right.y
  );
}

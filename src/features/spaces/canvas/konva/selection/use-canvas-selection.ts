import { shallowRef, watch, type Ref } from "vue";

import type { CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import type { CanvasSelectionSnapshot } from "../../contract/canvas-editor";
import { createCanvasSelectionSnapshot } from "../../document/canvas-selection-snapshot";
import { useCanvasMarqueeSelection } from "./use-canvas-marquee-selection";

interface CanvasSelectionOptions {
  availableStructureNodeIds: Readonly<Ref<ReadonlySet<string>>>;
  layouts: Readonly<Ref<readonly CanvasNodeLayoutV1[]>>;
  relationBounds: Readonly<Ref<ReadonlyMap<string, { x: number; y: number; width: number; height: number }>>>;
  changed(selection: CanvasSelectionSnapshot): void;
  structureSelectionForNode(nodeId: string): string | undefined;
}

export function useCanvasSelection(options: CanvasSelectionOptions) {
  const nodeIds = shallowRef(new Set<string>());
  const structureNodeIds = shallowRef(new Set<string>());
  const relationIds = shallowRef(new Set<string>());
  const marquee = useCanvasMarqueeSelection({
    clear,
    layouts: options.layouts,
    nodeIds,
    notify,
    structureNodeIds,
    structureSelectionForNode: options.structureSelectionForNode,
  });

  function selectNode(nodeId: string, event: PointerEvent) {
    const structureNodeId = options.structureSelectionForNode(nodeId);
    if (structureNodeId) {
      selectStructureNode(structureNodeId, event);
      return;
    }
    const additive = event.metaKey || event.ctrlKey || event.shiftKey;
    if (!additive && nodeIds.value.has(nodeId)) return;
    const next = new Set(additive ? nodeIds.value : []);
    if (next.has(nodeId) && additive) next.delete(nodeId);
    else next.add(nodeId);
    nodeIds.value = next;
    if (!additive) structureNodeIds.value = new Set();
    relationIds.value = new Set();
    notify();
  }

  function selectRelation(id: string, event: PointerEvent) {
    const additive = event.metaKey || event.ctrlKey || event.shiftKey;
    const next = new Set(additive ? relationIds.value : []);
    if (next.has(id) && additive) next.delete(id);
    else next.add(id);
    nodeIds.value = new Set();
    structureNodeIds.value = new Set();
    relationIds.value = next;
    notify();
  }

  function setNodeIds(ids: readonly string[]) {
    nodeIds.value = new Set(ids);
    structureNodeIds.value = new Set();
    relationIds.value = new Set();
    notify();
  }

  function setStructureNodeIds(ids: readonly string[]) {
    structureNodeIds.value = new Set(ids);
    nodeIds.value = new Set();
    relationIds.value = new Set();
    notify();
  }

  function clear() {
    nodeIds.value = new Set();
    structureNodeIds.value = new Set();
    relationIds.value = new Set();
    notify();
  }

  function notify() {
    options.changed({
      ...createCanvasSelectionSnapshot(
        options.layouts.value,
        nodeIds.value,
        relationIds.value,
        options.relationBounds.value,
      ),
      structureNodeIds: [...structureNodeIds.value],
    });
  }

  function selectStructureNode(structureNodeId: string, event: PointerEvent) {
    const additive = event.metaKey || event.ctrlKey || event.shiftKey;
    const next = new Set(additive ? structureNodeIds.value : []);
    if (next.has(structureNodeId) && additive) next.delete(structureNodeId);
    else next.add(structureNodeId);
    structureNodeIds.value = next;
    if (!additive) nodeIds.value = new Set();
    relationIds.value = new Set();
    notify();
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
  watch(options.availableStructureNodeIds, (availableIds) => {
    if (!structureNodeIds.value.size) return;
    const retained = new Set([...structureNodeIds.value].filter((id) => availableIds.has(id)));
    if (retained.size === structureNodeIds.value.size) return;
    structureNodeIds.value = retained;
    notify();
  });

  return {
    beginMarquee: marquee.begin,
    cancelMarquee: marquee.cancel,
    clear,
    endMarquee: marquee.end,
    marqueeRect: marquee.marqueeRect,
    moveMarquee: marquee.move,
    nodeIds,
    relationIds,
    selectNode,
    selectRelation,
    selectStructureNode,
    setNodeIds,
    setStructureNodeIds,
    structureNodeIds,
  };
}

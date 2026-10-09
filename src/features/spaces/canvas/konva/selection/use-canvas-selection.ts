import { compact } from "es-toolkit";
import { shallowRef, watch, type Ref } from "vue";

import type { CanvasRect, CanvasSelectionSnapshot } from "../../contracts/canvas-editor-port";
import { createCanvasSelectionSnapshot } from "../../document/canvas-selection-snapshot";
import type { CanvasNodeLayoutV1 } from "../../schema/document";
import { useCanvasMarqueeSelection } from "./use-canvas-marquee-selection";

interface CanvasSelectionOptions {
  availableStructureNodeIds: Readonly<Ref<ReadonlySet<string>>>;
  layouts: Readonly<Ref<readonly CanvasNodeLayoutV1[]>>;
  relationBoundsById: Readonly<
    Ref<ReadonlyMap<string, { x: number; y: number; width: number; height: number }>>
  >;
  structureBoundsById: Readonly<Ref<ReadonlyMap<string, CanvasRect>>>;
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
    if (event.button !== 0) return;
    const structureNodeId = options.structureSelectionForNode(nodeId);
    if (structureNodeId) {
      selectStructureNode(structureNodeId, event);
      return;
    }
    const additive = isAdditiveSelection(event);
    if (!additive && nodeIds.value.has(nodeId)) return;
    const next = new Set(additive ? nodeIds.value : []);
    if (next.has(nodeId) && additive) next.delete(nodeId);
    else next.add(nodeId);
    nodeIds.value = next;
    if (!additive) structureNodeIds.value = new Set();
    relationIds.value = new Set();
    notify();
  }
  function selectRelation(relationId: string, event: PointerEvent) {
    if (event.button !== 0) return;
    const additive = isAdditiveSelection(event);
    const next = new Set(additive ? relationIds.value : []);
    if (next.has(relationId) && additive) next.delete(relationId);
    else next.add(relationId);
    nodeIds.value = new Set();
    structureNodeIds.value = new Set();
    relationIds.value = next;
    notify();
  }
  function selectNodeForContextMenu(nodeId: string) {
    const structureNodeId = options.structureSelectionForNode(nodeId);
    if (structureNodeId) {
      selectStructureNodeForContextMenu(structureNodeId);
      return;
    }
    if (nodeIds.value.has(nodeId)) return;
    setNodeIds([nodeId]);
  }
  function selectRelationForContextMenu(relationId: string) {
    if (relationIds.value.has(relationId)) return;
    nodeIds.value = new Set();
    structureNodeIds.value = new Set();
    relationIds.value = new Set([relationId]);
    notify();
  }
  function setNodeIds(selectedNodeIds: readonly string[]) {
    nodeIds.value = new Set(selectedNodeIds);
    structureNodeIds.value = new Set();
    relationIds.value = new Set();
    notify();
  }
  function setStructureNodeIds(selectedStructureNodeIds: readonly string[]) {
    structureNodeIds.value = new Set(selectedStructureNodeIds);
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
    const snapshot = createCanvasSelectionSnapshot({
      layouts: options.layouts.value,
      nodeIds: nodeIds.value,
      relationIds: relationIds.value,
      relationBoundsById: options.relationBoundsById.value,
    });
    const bounds = selectedStructureBounds(structureNodeIds.value, options.structureBoundsById.value);
    options.changed({
      ...snapshot,
      structureNodeIds: [...structureNodeIds.value],
      ...(bounds ? { bounds } : {}),
    });
  }

  function selectStructureNode(structureNodeId: string, event: PointerEvent) {
    if (event.button !== 0) return;
    const additive = isAdditiveSelection(event);
    const next = new Set(additive ? structureNodeIds.value : []);
    if (next.has(structureNodeId) && additive) next.delete(structureNodeId);
    else next.add(structureNodeId);
    structureNodeIds.value = next;
    if (!additive) nodeIds.value = new Set();
    relationIds.value = new Set();
    notify();
  }

  function selectStructureNodeForContextMenu(structureNodeId: string) {
    if (structureNodeIds.value.has(structureNodeId)) return;
    setStructureNodeIds([structureNodeId]);
  }

  watch(options.layouts, (currentLayouts) => {
    if (!nodeIds.value.size) return;
    const visibleIds = new Set(currentLayouts.map(({ nodeId }) => nodeId));
    const retained = new Set([...nodeIds.value].filter((id) => visibleIds.has(id)));
    if (retained.size !== nodeIds.value.size) nodeIds.value = retained;
    notify();
  });
  watch(options.relationBoundsById, (currentBounds) => {
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
  watch(options.structureBoundsById, () => {
    if (structureNodeIds.value.size) notify();
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
    selectNodeForContextMenu,
    selectRelation,
    selectRelationForContextMenu,
    selectStructureNode,
    selectStructureNodeForContextMenu,
    setNodeIds,
    setStructureNodeIds,
    structureNodeIds,
  };
}

function selectedStructureBounds(
  structureNodeIds: ReadonlySet<string>,
  boundsByStructureNodeId: ReadonlyMap<string, CanvasRect>,
) {
  const frames = compact([...structureNodeIds].map((id) => boundsByStructureNodeId.get(id)));
  if (!frames.length) return undefined;
  const left = Math.min(...frames.map(({ x }) => x));
  const top = Math.min(...frames.map(({ y }) => y));
  const right = Math.max(...frames.map(({ x, width }) => x + width));
  const bottom = Math.max(...frames.map(({ y, height }) => y + height));
  return { x: left, y: top, width: right - left, height: bottom - top };
}

function isAdditiveSelection(event: PointerEvent) {
  return event.metaKey || event.ctrlKey || event.shiftKey;
}

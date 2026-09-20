import type Konva from "konva";
import { computed, shallowRef, watch, type Ref } from "vue";

import { indexStructure } from "../../../model/structure/structure-index";
import {
  structureOperationRoots,
  structureSelectionTarget,
} from "../../../organization/structure/structure-selection";
import type { CanvasNodeLayoutV1, SpaceEditorDocument } from "../../contract/canvas-document";
import type { CanvasRect, CanvasSelectionSnapshot } from "../../contract/canvas-editor-port";
import { createCanvasSelectionSnapshot } from "../../document/canvas-selection-snapshot";
import { transformedStructureBounds } from "../../document/structure-transform";
interface SelectionOptions {
  document(): SpaceEditorDocument;
  scopeGroupId(): string | null;
  layouts: Readonly<Ref<readonly CanvasNodeLayoutV1[]>>;
  relationBounds: Readonly<Ref<ReadonlyMap<string, CanvasRect>>>;
  changed(selection: CanvasSelectionSnapshot): void;
}
export function useCanvasSelection(options: SelectionOptions) {
  const nodeIds = shallowRef(new Set<string>()),
    relationIds = shallowRef(new Set<string>());
  const marquee = shallowRef<{ start: { x: number; y: number }; end: { x: number; y: number } }>();
  const marqueeRect = computed(() =>
    marquee.value
      ? {
          x: Math.min(marquee.value.start.x, marquee.value.end.x),
          y: Math.min(marquee.value.start.y, marquee.value.end.y),
          width: Math.abs(marquee.value.end.x - marquee.value.start.x),
          height: Math.abs(marquee.value.end.y - marquee.value.start.y),
        }
      : undefined,
  );
  function selectNode(id: string, event: PointerEvent) {
    const additive = event.ctrlKey || event.metaKey || event.shiftKey;
    if (!additive && nodeIds.value.has(id)) return;
    const selected = new Set(additive ? nodeIds.value : []);
    if (additive && selected.has(id)) selected.delete(id);
    else selected.add(id);
    setNodeIds([...selected]);
  }
  function selectRelation(id: string, event: PointerEvent) {
    const additive = event.ctrlKey || event.metaKey || event.shiftKey;
    const selected = new Set(additive ? relationIds.value : []);
    if (additive && selected.has(id)) selected.delete(id);
    else selected.add(id);
    nodeIds.value = new Set();
    relationIds.value = selected;
    notify();
  }
  function beginMarquee(event: Konva.KonvaEventObject<PointerEvent>) {
    if (event.evt.button !== 0 || event.target !== event.target.getStage()) return false;
    const point = event.target.getStage()?.getRelativePointerPosition();
    if (!point) return false;
    marquee.value = { start: point, end: point };
    if (!(event.evt.ctrlKey || event.evt.metaKey || event.evt.shiftKey)) clear();
    return true;
  }
  function moveMarquee(event: Konva.KonvaEventObject<PointerEvent>) {
    const point = event.target.getStage()?.getRelativePointerPosition();
    if (marquee.value && point) marquee.value = { ...marquee.value, end: point };
  }
  function endMarquee() {
    const bounds = marqueeRect.value;
    marquee.value = undefined;
    if (!bounds || bounds.width < 1 || bounds.height < 1) return;
    const tree = indexStructure(options.document().structure);
    const selected = new Set(nodeIds.value);
    for (const layout of options.layouts.value) {
      const item = tree.items.get(layout.nodeId)!;
      if (item.kind === "group") continue;
      const frame = transformedStructureBounds({ ...layout.frame, scale: 1 }, layout.frame);
      const inside =
        frame.x >= bounds.x &&
        frame.y >= bounds.y &&
        frame.x + frame.width <= bounds.x + bounds.width &&
        frame.y + frame.height <= bounds.y + bounds.height;
      const overlap =
        frame.x <= bounds.x + bounds.width &&
        frame.y <= bounds.y + bounds.height &&
        frame.x + frame.width >= bounds.x &&
        frame.y + frame.height >= bounds.y;
      if (!(item.kind === "section" ? inside : overlap)) continue;
      const target = structureSelectionTarget(tree, item.id, options.scopeGroupId());
      if (target) selected.add(target);
    }
    setNodeIds([...selected]);
  }
  function setNodeIds(ids: readonly string[]) {
    const tree = indexStructure(options.document().structure);
    nodeIds.value = new Set(
      structureOperationRoots(
        tree,
        ids.filter((id) => tree.items.has(id)),
      ).map((item) => item.id),
    );
    relationIds.value = new Set();
    notify();
  }
  function clear() {
    nodeIds.value = new Set();
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
      scopeGroupId: options.scopeGroupId(),
    });
  }
  watch(options.layouts, (layouts) => {
    const visible = new Set(layouts.map((layout) => layout.nodeId));
    nodeIds.value = new Set([...nodeIds.value].filter((id) => visible.has(id)));
    notify();
  });
  watch(options.relationBounds, (bounds) => {
    relationIds.value = new Set([...relationIds.value].filter((id) => bounds.has(id)));
    notify();
  });
  return {
    nodeIds,
    relationIds,
    marqueeRect,
    selectNode,
    selectRelation,
    setNodeIds,
    clear,
    beginMarquee,
    moveMarquee,
    endMarquee,
    cancelMarquee: () => {
      marquee.value = undefined;
    },
  };
}

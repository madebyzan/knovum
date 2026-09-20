import type { SpaceEditorDocument } from "../../../canvas/contract/canvas-document";
import type { CanvasLayerMove } from "../../../canvas/contract/canvas-editor-port";
import { indexStructure } from "../../../model/structure/structure-index";
import { assertStructureEditable, structureOperationRoots } from "../structure-selection";

export function reorderStructureItems(
  document: SpaceEditorDocument,
  ids: readonly string[],
  mode: CanvasLayerMove,
): SpaceEditorDocument {
  const tree = indexStructure(document.structure);
  const roots = structureOperationRoots(tree, ids);
  assertStructureEditable(
    tree,
    roots.map((item) => item.id),
  );
  const selected = new Set(roots.map((item) => item.id));
  const orders = new Map<string, number>();
  for (const parentId of new Set(roots.map((item) => item.parentId))) {
    const siblings = (tree.children.get(parentId) ?? []).map((item) => item.id);
    reorder(siblings, selected, mode).forEach((id, order) => orders.set(id, order));
  }
  return {
    ...document,
    structure: {
      version: 1,
      items: document.structure.items.map((item) =>
        orders.has(item.id) ? { ...item, order: orders.get(item.id)! } : item,
      ),
    },
  };
}
function reorder(ids: string[], selected: ReadonlySet<string>, mode: CanvasLayerMove) {
  const chosen = ids.filter((id) => selected.has(id));
  const others = ids.filter((id) => !selected.has(id));
  if (mode === "front") return [...others, ...chosen];
  if (mode === "back") return [...chosen, ...others];
  const direction = mode === "forward" ? -1 : 1;
  for (let index = direction < 0 ? ids.length - 2 : 1; index >= 0 && index < ids.length; index += direction) {
    const other = index - direction;
    if (selected.has(ids[index]!) && !selected.has(ids[other]!))
      [ids[index], ids[other]] = [ids[other]!, ids[index]!];
  }
  return ids;
}

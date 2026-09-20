import type { StructureItemLayout } from "../../canvas/contract/structure-layout";
import { StructureError } from "../../model/structure/structure-error";
import { indexStructure, type StructureIndex } from "../../model/structure/structure-index";
import type { StructureSnapshot } from "./structure-snapshot-contract";

export interface StructureContext {
  readonly tree: StructureIndex;
  readonly layouts: ReadonlyMap<string, StructureItemLayout>;
}
export function indexStructureSnapshot(snapshot: StructureSnapshot): StructureContext {
  const tree = indexStructure(snapshot.structure);
  const layouts = new Map<string, StructureItemLayout>();
  for (const layout of snapshot.layout.items) {
    if (layouts.has(layout.itemId))
      throw new StructureError("duplicate-identity", "Layout identities must be unique", [layout.itemId]);
    if (tree.items.get(layout.itemId)?.kind !== layout.kind)
      throw new StructureError("invalid-layout", "Layout kind must match an existing structure item", [
        layout.itemId,
      ]);
    layouts.set(layout.itemId, layout);
  }
  for (const item of tree.items.values())
    if (!layouts.has(item.id))
      throw new StructureError("invalid-layout", "Every structure item requires one layout", [item.id]);
  return { tree, layouts };
}
export function requireStructureLayout(context: StructureContext, id: string) {
  const layout = context.layouts.get(id);
  if (!layout) throw new StructureError("invalid-layout", "Item layout is unavailable", [id]);
  return layout;
}

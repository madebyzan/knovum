import { StructureError } from "./structure-error";
import type { SpaceStructure, StructureItem } from "./structure-schema";

export interface StructureIndex {
  readonly items: ReadonlyMap<string, StructureItem>;
  readonly children: ReadonlyMap<string | null, readonly StructureItem[]>;
}
export function indexStructure(structure: SpaceStructure): StructureIndex {
  const items = new Map<string, StructureItem>();
  const children = new Map<string | null, StructureItem[]>();
  for (const item of structure.items) {
    if (items.has(item.id))
      throw new StructureError("duplicate-identity", "Structure identities must be unique", [item.id]);
    items.set(item.id, item);
    const siblings = children.get(item.parentId) ?? [];
    siblings.push(item);
    children.set(item.parentId, siblings);
  }
  for (const [parent, siblings] of children)
    children.set(
      parent,
      siblings.toSorted((a, b) => a.order - b.order),
    );
  return { items, children };
}
export function requireStructureItem(index: StructureIndex, id: string): StructureItem {
  const item = index.items.get(id);
  if (!item) throw new StructureError("missing-item", "The structure item is unavailable", [id]);
  return item;
}
/** Queries accept topology validated at the document boundary. */
export function structureAncestors(index: StructureIndex, id: string): StructureItem[] {
  const ancestors: StructureItem[] = [];
  let parentId = requireStructureItem(index, id).parentId;
  while (parentId !== null) {
    const parent = requireStructureItem(index, parentId);
    ancestors.push(parent);
    parentId = parent.parentId;
  }
  return ancestors;
}
export function structureSubtree(index: StructureIndex, roots: readonly string[]): StructureItem[] {
  const pending = roots.toReversed();
  const seen = new Set<string>();
  const result: StructureItem[] = [];
  while (pending.length) {
    const id = pending.pop()!;
    if (seen.has(id)) continue;
    seen.add(id);
    const item = requireStructureItem(index, id);
    result.push(item);
    for (const child of (index.children.get(id) ?? []).toReversed()) pending.push(child.id);
  }
  return result;
}

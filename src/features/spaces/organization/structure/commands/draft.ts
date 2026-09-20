import type { StructureItemLayout, StructureTransform } from "../../../canvas/contract/structure-layout";
import { StructureError } from "../../../model/structure/structure-error";
import type { StructureItem } from "../../../model/structure/structure-schema";
import type { StructureSnapshot } from "../structure-snapshot-contract";

export interface StructureDraft {
  items: Map<string, StructureItem>;
  layouts: Map<string, StructureItemLayout>;
}
export function createStructureDraft(snapshot: StructureSnapshot): StructureDraft {
  return { items: new Map(snapshot.structure.items.map((item) => [item.id, item])), layouts: new Map(snapshot.layout.items.map((item) => [item.itemId, item])) };
}
export function draftSnapshot(draft: StructureDraft): StructureSnapshot {
  return { structure: { version: 1, items: [...draft.items.values()].toSorted((a, b) => compare(a.id, b.id)) },
    layout: { version: 1, items: [...draft.layouts.values()].toSorted((a, b) => compare(a.itemId, b.itemId)) } };
}
export function requireDraftItem(draft: StructureDraft, id: string) {
  const item = draft.items.get(id);
  if (!item) throw new StructureError("missing-item", "The item is unavailable", [id]);
  return item;
}
export function assertNewIdentity(draft: StructureDraft, id: string) {
  if (draft.items.has(id) || draft.layouts.has(id)) throw new StructureError("duplicate-identity", "A new item cannot replace an existing identity", [id]);
}
export function draftSiblings(draft: StructureDraft, parentId: string | null) {
  return [...draft.items.values()].filter((item) => item.parentId === parentId).toSorted((a, b) => a.order - b.order).map((item) => item.id);
}
export function setDraftSiblings(draft: StructureDraft, parentId: string | null, ids: readonly string[]) {
  for (const [order, id] of ids.entries()) {
    const item = requireDraftItem(draft, id);
    if (item.parentId !== parentId || item.order !== order) draft.items.set(id, { ...item, parentId, order });
  }
}
export function setDraftTransform(draft: StructureDraft, id: string, transform: StructureTransform) {
  const layout = draft.layouts.get(id);
  if (!layout) throw new StructureError("invalid-layout", "Item geometry is unavailable", [id]);
  if (layout.kind !== "section") draft.layouts.set(id, { ...layout, transform });
  else {
    if (transform.rotation !== 0 || transform.scale !== 1) throw new StructureError("invalid-layout", "Sections cannot rotate or scale their contents", [id]);
    draft.layouts.set(id, { ...layout, position: { x: transform.x, y: transform.y } });
  }
}
export function removeDraftItem(draft: StructureDraft, id: string) { draft.items.delete(id); draft.layouts.delete(id); }
export function normalizeDraftOrders(draft: StructureDraft, parents: ReadonlySet<string | null>) {
  const buckets = new Map<string | null, StructureItem[]>();
  for (const item of draft.items.values()) {
    if (!parents.has(item.parentId)) continue;
    const bucket = buckets.get(item.parentId) ?? [];
    bucket.push(item); buckets.set(item.parentId, bucket);
  }
  for (const [parent, items] of buckets) setDraftSiblings(draft, parent, items.toSorted((a, b) => a.order - b.order).map((item) => item.id));
}
/** Empty Groups are removed by their causing command, never while loading a document. */
export function pruneEmptyGroups(draft: StructureDraft) {
  const counts = new Map<string | null, number>();
  for (const item of draft.items.values()) counts.set(item.parentId, (counts.get(item.parentId) ?? 0) + 1);
  const pending = [...draft.items.values()].filter((item) => item.kind === "group" && !counts.has(item.id));
  const affected = new Set<string | null>();
  while (pending.length) {
    const group = pending.pop()!;
    removeDraftItem(draft, group.id); affected.add(group.parentId);
    const count = counts.get(group.parentId)! - 1;
    counts.set(group.parentId, count);
    const parent = group.parentId === null ? undefined : draft.items.get(group.parentId);
    if (count === 0 && parent?.kind === "group") pending.push(parent);
  }
  normalizeDraftOrders(draft, affected);
}
function compare(a: string, b: string) { return a < b ? -1 : a > b ? 1 : 0; }

import { StructureError } from "./structure-error";
import type { StructureIndex } from "./structure-index";
import { MAX_STRUCTURE_DEPTH, type StructureItem } from "./structure-schema";

export function validateStructure(index: StructureIndex) {
  const contentIds = new Set<string>();
  for (const item of index.items.values()) {
    validateParent(index, item);
    if (item.kind === "content") {
      if (item.id !== item.nodeId || contentIds.has(item.nodeId)) {
        throw new StructureError(
          "invalid-reference",
          "Each content identity has exactly one placement using its own identity",
          [item.id],
        );
      }
      contentIds.add(item.nodeId);
    }
    if (item.kind === "group" && !index.children.get(item.id)?.length) {
      throw new StructureError("empty-group", "An empty Group cannot be committed", [item.id]);
    }
  }
  validateOrder(index);
  validateDepth(index);
}
function validateParent(index: StructureIndex, item: StructureItem) {
  if (item.kind === "section" && item.parentId !== null) {
    throw new StructureError("invalid-parent", "Sections belong to the Space root", [item.id]);
  }
  if (item.parentId !== null) {
    const parent = index.items.get(item.parentId);
    if (!parent || parent.kind === "content")
      throw new StructureError("invalid-parent", "A parent must be an existing container", [item.id]);
  }
}
function validateOrder(index: StructureIndex) {
  for (const siblings of index.children.values())
    for (const [order, item] of siblings.entries()) {
      if (item.order !== order)
        throw new StructureError("invalid-order", "Sibling order must be unique and contiguous", [item.id]);
    }
}
function validateDepth(index: StructureIndex) {
  const pending = (index.children.get(null) ?? []).map((item) => ({ id: item.id, depth: 1 }));
  let visited = 0;
  while (pending.length) {
    const { id, depth } = pending.pop()!;
    if (depth > MAX_STRUCTURE_DEPTH)
      throw new StructureError("depth-limit", "Structure nesting exceeds its limit", [id]);
    visited += 1;
    for (const child of index.children.get(id) ?? []) pending.push({ id: child.id, depth: depth + 1 });
  }
  if (visited !== index.items.size) throw new StructureError("cycle", "Structure contains a parent cycle");
}
export function validateContentPlacements(index: StructureIndex, nodeIds: ReadonlySet<string>) {
  const missing = new Set(nodeIds);
  for (const item of index.items.values()) {
    if (item.kind === "content" && !missing.delete(item.nodeId)) {
      throw new StructureError(
        "invalid-reference",
        "A placement must reference active content in this Space",
        [item.id],
      );
    }
  }
  if (missing.size)
    throw new StructureError("invalid-reference", "Every active content item requires a placement", [
      ...missing,
    ]);
}

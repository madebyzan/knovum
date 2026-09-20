import { IDENTITY_STRUCTURE_TRANSFORM } from "../../../canvas/document/structure-transform";
import { StructureError } from "../../../model/structure/structure-error";
import { requireStructureItem } from "../../../model/structure/structure-index";
import type { StructureItem } from "../../../model/structure/structure-schema";
import type { StructureContext } from "../structure-context";
import type { StructureProjection } from "../structure-projection";
import { assertStructureEditable, requireStructureRoots } from "../structure-selection";
import type { StructureDraft } from "./draft";

export interface StructureOperation {
  context: StructureContext;
  projection: StructureProjection;
  draft: StructureDraft;
  selection: readonly string[];
}
export interface StructureOperationResult {
  selection: readonly string[];
  contentClones?: readonly { sourceNodeId: string; targetNodeId: string }[];
}
export function operationRoots(operation: StructureOperation, ids: readonly string[]) {
  return requireStructureRoots(operation.context.tree, ids).toSorted(
    (a, b) => operation.projection.paintOrder.get(a.id)! - operation.projection.paintOrder.get(b.id)!,
  );
}
export function sharedParent(items: readonly StructureItem[]) {
  const parentId = items[0]!.parentId;
  if (items.some((item) => item.parentId !== parentId))
    throw new StructureError(
      "different-parents",
      "Move items into the same parent before combining them",
      items.map((item) => item.id),
    );
  return parentId;
}
export function requireEditableParent(operation: StructureOperation, id: string | null) {
  if (id === null) return;
  if (requireStructureItem(operation.context.tree, id).kind === "content")
    throw new StructureError("invalid-parent", "Content cannot contain other items", [id]);
  assertStructureEditable(operation.context.tree, [id]);
}
export function parentWorld(operation: StructureOperation, id: string | null) {
  if (id === null) return IDENTITY_STRUCTURE_TRANSFORM;
  requireStructureItem(operation.context.tree, id);
  return operation.projection.transforms.get(id)!;
}
export function insertionIndex(index: number | undefined, length: number) {
  const resolved = index ?? length;
  if (resolved > length)
    throw new StructureError("invalid-order", "Insertion position exceeds the remaining sibling list");
  return resolved;
}

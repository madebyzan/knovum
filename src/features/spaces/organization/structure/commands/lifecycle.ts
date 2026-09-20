import { StructureError } from "../../../model/structure/structure-error";
import {
  requireStructureItem,
  structureAncestors,
  structureSubtree,
} from "../../../model/structure/structure-index";
import { assertStructureEditable } from "../structure-selection";
import type { StructureCommandOf } from "./contract";
import { normalizeDraftOrders, pruneEmptyGroups, removeDraftItem } from "./draft";
import { operationRoots, type StructureOperation, type StructureOperationResult } from "./operation";

export function renameContainer(
  operation: StructureOperation,
  command: StructureCommandOf<"rename-container">,
): StructureOperationResult {
  const item = requireStructureItem(operation.context.tree, command.id);
  if (item.kind === "content")
    throw new StructureError("invalid-selection", "Content titles belong to the content editor", [item.id]);
  assertStructureEditable(operation.context.tree, [item.id]);
  operation.draft.items.set(item.id, { ...item, title: command.title });
  return { selection: operation.selection };
}
export function setItemsLocked(
  operation: StructureOperation,
  command: StructureCommandOf<"set-items-locked">,
): StructureOperationResult {
  const roots = operationRoots(operation, command.ids);
  const blocked = roots.filter((item) =>
    structureAncestors(operation.context.tree, item.id).some((parent) => parent.locked),
  );
  if (blocked.length)
    throw new StructureError(
      "locked",
      "Unlock the ancestor before changing a descendant lock",
      blocked.map((item) => item.id),
    );
  for (const item of roots) operation.draft.items.set(item.id, { ...item, locked: command.locked });
  return { selection: roots.map((item) => item.id) };
}
export function deleteSubtrees(
  operation: StructureOperation,
  command: StructureCommandOf<"delete-subtrees">,
): StructureOperationResult {
  const roots = operationRoots(operation, command.ids);
  assertStructureEditable(
    operation.context.tree,
    roots.map((item) => item.id),
  );
  const subtree = structureSubtree(
    operation.context.tree,
    roots.map((item) => item.id),
  );
  const protectedItems = subtree.filter((item) => item.locked);
  if (protectedItems.length)
    throw new StructureError(
      "locked",
      "Unlock protected descendants before deleting this operation unit",
      protectedItems.map((item) => item.id),
    );
  for (const item of subtree) removeDraftItem(operation.draft, item.id);
  normalizeDraftOrders(operation.draft, new Set(roots.map((item) => item.parentId)));
  pruneEmptyGroups(operation.draft);
  return { selection: operation.selection.filter((id) => operation.draft.items.has(id)) };
}

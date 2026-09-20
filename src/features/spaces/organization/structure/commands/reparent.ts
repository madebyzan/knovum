import type { StructurePoint } from "../../../canvas/contract/structure-layout";
import {
  composeStructureTransforms,
  invertStructureTransform,
} from "../../../canvas/document/structure-transform";
import { StructureError } from "../../../model/structure/structure-error";
import { structureSubtree } from "../../../model/structure/structure-index";
import { assertStructureEditable } from "../structure-selection";
import type { StructureCommandOf } from "./contract";
import {
  draftSiblings,
  normalizeDraftOrders,
  pruneEmptyGroups,
  setDraftSiblings,
  setDraftTransform,
} from "./draft";
import { moveItems } from "./movement";
import {
  insertionIndex,
  operationRoots,
  parentWorld,
  requireEditableParent,
  type StructureOperation,
  type StructureOperationResult,
} from "./operation";

export function reparentItems(
  operation: StructureOperation,
  command: StructureCommandOf<"reparent-items">,
): StructureOperationResult {
  return relocate(operation, { ...command, delta: { x: 0, y: 0 } });
}
export function dropItems(
  operation: StructureOperation,
  command: StructureCommandOf<"drop-items">,
): StructureOperationResult {
  const roots = operationRoots(operation, command.ids);
  if (roots.every((item) => item.parentId === command.parentId))
    return moveItems(operation, { kind: "move-items", ids: command.ids, delta: command.delta });
  return relocate(operation, command);
}
function relocate(
  operation: StructureOperation,
  command: {
    ids: readonly string[];
    parentId: string | null;
    delta: StructurePoint;
    insertIndex?: number | undefined;
  },
): StructureOperationResult {
  const roots = operationRoots(operation, command.ids);
  const ids = roots.map((item) => item.id);
  assertStructureEditable(operation.context.tree, ids);
  requireEditableParent(operation, command.parentId);
  if (command.parentId !== null) {
    if (roots.some((item) => item.kind === "section"))
      throw new StructureError("invalid-parent", "Sections belong to the Space root");
    if (structureSubtree(operation.context.tree, ids).some((item) => item.id === command.parentId))
      throw new StructureError("cycle", "An item cannot contain itself or its ancestor");
  }
  const selected = new Set(ids);
  const siblings = draftSiblings(operation.draft, command.parentId).filter((id) => !selected.has(id));
  const index = insertionIndex(command.insertIndex, siblings.length);
  const inverse = invertStructureTransform(parentWorld(operation, command.parentId));
  for (const item of roots) {
    const world = operation.projection.transforms.get(item.id)!;
    const moved = { ...world, x: world.x + command.delta.x, y: world.y + command.delta.y };
    setDraftTransform(operation.draft, item.id, composeStructureTransforms(inverse, moved));
    operation.draft.items.set(item.id, { ...item, parentId: command.parentId });
  }
  const oldParents = new Set(roots.map((item) => item.parentId));
  oldParents.delete(command.parentId);
  normalizeDraftOrders(operation.draft, oldParents);
  setDraftSiblings(
    operation.draft,
    command.parentId,
    siblings.slice(0, index).concat(ids, siblings.slice(index)),
  );
  pruneEmptyGroups(operation.draft);
  return { selection: ids };
}

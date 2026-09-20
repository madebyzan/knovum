import type { StructurePoint } from "../../../canvas/contract/structure-layout";
import { composeStructureTransforms, invertStructureTransform } from "../../../canvas/document/structure-transform";
import { StructureError } from "../../../model/structure/structure-error";
import { requireStructureItem } from "../../../model/structure/structure-index";
import type { StructureItem } from "../../../model/structure/structure-schema";
import { assertStructureEditable } from "../structure-selection";
import type { StructureCommandOf } from "./contract";
import { setDraftTransform } from "./draft";
import { operationRoots, parentWorld, type StructureOperation, type StructureOperationResult } from "./operation";

export function moveItems(operation: StructureOperation, command: StructureCommandOf<"move-items">): StructureOperationResult {
  const roots = operationRoots(operation, command.ids);
  assertStructureEditable(operation.context.tree, roots.map((item) => item.id));
  for (const root of roots) moveRoot(operation, root, command.delta);
  return { selection: roots.map((item) => item.id) };
}
export function moveRoot(operation: StructureOperation, root: StructureItem, delta: StructurePoint) {
  if (delta.x === 0 && delta.y === 0) return;
  const world = operation.projection.transforms.get(root.id)!;
  const moved = { ...world, x: world.x + delta.x, y: world.y + delta.y };
  setDraftTransform(operation.draft, root.id, composeStructureTransforms(invertStructureTransform(parentWorld(operation, root.parentId)), moved));
}
export function transformGroup(operation: StructureOperation, command: StructureCommandOf<"transform-group">): StructureOperationResult {
  const item = requireStructureItem(operation.context.tree, command.id);
  if (item.kind !== "group") throw new StructureError("invalid-selection", "Aggregate rotation and scaling require a Group", [item.id]);
  assertStructureEditable(operation.context.tree, [item.id]);
  const local = composeStructureTransforms(invertStructureTransform(parentWorld(operation, item.parentId)), command.worldTransform);
  setDraftTransform(operation.draft, item.id, local);
  return { selection: [item.id] };
}

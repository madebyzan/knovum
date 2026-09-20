import {
  composeStructureTransforms,
  invertStructureTransform,
  structureLayoutTransform,
} from "../../../canvas/document/structure-transform";
import { StructureError } from "../../../model/structure/structure-error";
import { requireStructureItem } from "../../../model/structure/structure-index";
import type { StructureItem } from "../../../model/structure/structure-schema";
import { requireStructureLayout } from "../structure-context";
import { structureBoundsInParent } from "../structure-projection";
import { assertStructureEditable } from "../structure-selection";
import type { StructureCommandOf } from "./contract";
import {
  assertNewIdentity,
  draftSiblings,
  removeDraftItem,
  setDraftSiblings,
  setDraftTransform,
} from "./draft";
import {
  operationRoots,
  parentWorld,
  sharedParent,
  type StructureOperation,
  type StructureOperationResult,
} from "./operation";

export function groupSelection(
  operation: StructureOperation,
  command: StructureCommandOf<"group-selection">,
): StructureOperationResult {
  const roots = operationRoots(operation, command.ids);
  if (roots.length < 2 || roots.some((item) => item.kind === "section"))
    throw new StructureError("invalid-selection", "Combine at least two sibling content items or Groups");
  const parentId = sharedParent(roots);
  assertStructureEditable(
    operation.context.tree,
    roots.map((item) => item.id),
  );
  assertNewIdentity(operation.draft, command.id);
  const bounds = structureBoundsInParent(
    operation.context,
    operation.projection,
    roots.map((item) => item.id),
    parentId,
  )!;
  const transform = { x: bounds.x, y: bounds.y, rotation: 0, scale: 1 };
  const siblings = replacementSiblings(operation, roots, command.id);
  operation.draft.items.set(command.id, {
    kind: "group",
    id: command.id,
    title: command.title,
    parentId,
    order: 0,
    locked: false,
  });
  operation.draft.layouts.set(command.id, { kind: "group", itemId: command.id, transform });
  const inverse = invertStructureTransform(transform);
  for (const root of roots) {
    setDraftTransform(
      operation.draft,
      root.id,
      composeStructureTransforms(
        inverse,
        structureLayoutTransform(requireStructureLayout(operation.context, root.id)),
      ),
    );
  }
  setDraftSiblings(
    operation.draft,
    command.id,
    roots.map((item) => item.id),
  );
  setDraftSiblings(operation.draft, parentId, siblings);
  return { selection: [command.id] };
}
export function replacementSiblings(
  operation: StructureOperation,
  roots: readonly StructureItem[],
  id: string,
) {
  const parentId = sharedParent(roots);
  const chosen = new Set(roots.map((item) => item.id));
  const front = roots.reduce((order, item) => Math.max(order, item.order), -1);
  const other = (operation.context.tree.children.get(parentId) ?? []).filter((item) => !chosen.has(item.id));
  const insertion = other.filter((item) => item.order < front).length;
  const siblings = other.map((item) => item.id);
  siblings.splice(insertion, 0, id);
  return siblings;
}
export function dissolveContainer(
  operation: StructureOperation,
  command: StructureCommandOf<"dissolve-container">,
): StructureOperationResult {
  const item = requireStructureItem(operation.context.tree, command.id);
  if (item.kind === "content")
    throw new StructureError("invalid-selection", "Only containers can be dissolved", [item.id]);
  assertStructureEditable(operation.context.tree, [item.id]);
  const children = operation.context.tree.children.get(item.id) ?? [];
  const inverse = invertStructureTransform(parentWorld(operation, item.parentId));
  for (const child of children)
    setDraftTransform(
      operation.draft,
      child.id,
      composeStructureTransforms(inverse, operation.projection.transforms.get(child.id)!),
    );
  const siblings = draftSiblings(operation.draft, item.parentId);
  removeDraftItem(operation.draft, item.id);
  setDraftSiblings(
    operation.draft,
    item.parentId,
    siblings.slice(0, item.order).concat(
      children.map((child) => child.id),
      siblings.slice(item.order + 1),
    ),
  );
  return { selection: children.map((child) => child.id) };
}

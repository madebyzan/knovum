import type { StructureBounds } from "../../../canvas/contract/structure-layout";
import { composeStructureTransforms, invertStructureTransform } from "../../../canvas/document/structure-transform";
import { StructureError } from "../../../model/structure/structure-error";
import type { StructureItem } from "../../../model/structure/structure-schema";
import { structureBoundsInParent } from "../structure-projection";
import { assertStructureEditable, structureOperationRoots } from "../structure-selection";
import type { StructureCommandOf } from "./contract";
import { assertNewIdentity, draftSiblings, normalizeDraftOrders, setDraftSiblings, setDraftTransform } from "./draft";
import { replacementSiblings } from "./grouping";
import { operationRoots, sharedParent, type StructureOperation, type StructureOperationResult } from "./operation";

export const SECTION_INSETS = Object.freeze({ left: 32, top: 56, right: 32, bottom: 32 });
export function sectionBoundsAround(bounds: StructureBounds): StructureBounds {
  return { x: bounds.x - SECTION_INSETS.left, y: bounds.y - SECTION_INSETS.top,
    width: bounds.width + SECTION_INSETS.left + SECTION_INSETS.right,
    height: bounds.height + SECTION_INSETS.top + SECTION_INSETS.bottom };
}
export function createEmptySection(operation: StructureOperation, command: StructureCommandOf<"create-empty-section">): StructureOperationResult {
  return createWithMembers(operation, command, command.bounds, []);
}
export function drawSection(operation: StructureOperation, command: StructureCommandOf<"draw-section">): StructureOperationResult {
  const roots = structureOperationRoots(operation.context.tree, command.ids).toSorted((a, b) => a.order - b.order);
  if (roots.some((item) => item.parentId !== null || item.kind === "section")) throw new StructureError("invalid-selection", "Drawn Sections adopt only root-level content or Groups");
  assertStructureEditable(operation.context.tree, roots.map((item) => item.id));
  if (roots.some((item) => !containsBounds(command.bounds, operation.projection.bounds.get(item.id)!))) throw new StructureError("invalid-selection", "A drawn Section must fully enclose each adopted operation unit");
  return createWithMembers(operation, command, command.bounds, roots);
}
export function sectionAroundSelection(operation: StructureOperation, command: StructureCommandOf<"section-around-selection">): StructureOperationResult {
  const roots = operationRoots(operation, command.ids);
  const parentId = sharedParent(roots);
  if (roots.some((item) => item.kind === "section") || (parentId !== null && operation.context.tree.items.get(parentId)?.kind !== "section")) {
    throw new StructureError("invalid-selection", "Create a Section around sibling content or Groups at the root or inside one Section");
  }
  assertStructureEditable(operation.context.tree, roots.map((item) => item.id));
  const bounds = structureBoundsInParent(operation.context, operation.projection, roots.map((item) => item.id), null)!;
  return createWithMembers(operation, command, sectionBoundsAround(bounds), roots);
}
function createWithMembers(operation: StructureOperation, identity: { id: string; title: string }, bounds: StructureBounds, roots: readonly StructureItem[]): StructureOperationResult {
  assertNewIdentity(operation.draft, identity.id);
  const sourceParent = roots.length ? sharedParent(roots) : null;
  const siblings = roots.length && sourceParent === null ? replacementSiblings(operation, roots, identity.id) : draftSiblings(operation.draft, null);
  if (!roots.length) siblings.push(identity.id);
  else if (sourceParent !== null) siblings.splice(siblings.indexOf(sourceParent) + 1, 0, identity.id);
  operation.draft.items.set(identity.id, { kind: "section", id: identity.id, title: identity.title, parentId: null, order: 0, locked: false });
  operation.draft.layouts.set(identity.id, { kind: "section", itemId: identity.id, position: { x: bounds.x, y: bounds.y }, size: { width: bounds.width, height: bounds.height }, collapsed: false });
  const inverse = invertStructureTransform({ x: bounds.x, y: bounds.y, rotation: 0, scale: 1 });
  for (const root of roots) setDraftTransform(operation.draft, root.id, composeStructureTransforms(inverse, operation.projection.transforms.get(root.id)!));
  setDraftSiblings(operation.draft, identity.id, roots.map((item) => item.id));
  if (sourceParent !== null) normalizeDraftOrders(operation.draft, new Set([sourceParent]));
  setDraftSiblings(operation.draft, null, siblings);
  return { selection: [identity.id] };
}
function containsBounds(outer: StructureBounds, inner: StructureBounds) {
  return inner.x >= outer.x && inner.y >= outer.y && inner.x + inner.width <= outer.x + outer.width && inner.y + inner.height <= outer.y + outer.height;
}

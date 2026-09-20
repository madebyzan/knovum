import { composeStructureTransforms, invertStructureTransform } from "../../../canvas/document/structure-transform";
import { StructureError } from "../../../model/structure/structure-error";
import { structureSubtree } from "../../../model/structure/structure-index";
import type { StructureItem } from "../../../model/structure/structure-schema";
import { requireStructureLayout } from "../structure-context";
import type { StructureCommandOf } from "./contract";
import { assertNewIdentity, draftSiblings, setDraftSiblings, setDraftTransform } from "./draft";
import { insertionIndex, operationRoots, parentWorld, requireEditableParent, type StructureOperation, type StructureOperationResult } from "./operation";

export function cloneSubtrees(operation: StructureOperation, command: StructureCommandOf<"clone-subtrees">): StructureOperationResult {
  const roots = operationRoots(operation, command.ids);
  requireEditableParent(operation, command.parentId);
  if (command.parentId !== null && roots.some((item) => item.kind === "section")) throw new StructureError("invalid-parent", "Copied Sections must remain at the Space root");
  const source = structureSubtree(operation.context.tree, roots.map((item) => item.id));
  const identities = validateIdentities(operation, source, command.identities);
  const rootIds = new Set(roots.map((item) => item.id));
  const siblings = draftSiblings(operation.draft, command.parentId);
  const index = insertionIndex(command.insertIndex, siblings.length);
  const contentClones: { sourceNodeId: string; targetNodeId: string }[] = [];
  for (const item of source) {
    const id = identities.get(item.id)!;
    const parentId = rootIds.has(item.id) ? command.parentId : identities.get(item.parentId!)!;
    const clone = item.kind === "content" ? { ...item, id, nodeId: id, parentId } : { ...item, id, parentId };
    operation.draft.items.set(id, clone);
    operation.draft.layouts.set(id, { ...structuredClone(requireStructureLayout(operation.context, item.id)), itemId: id });
    if (item.kind === "content") contentClones.push({ sourceNodeId: item.nodeId, targetNodeId: id });
  }
  const inverse = invertStructureTransform(parentWorld(operation, command.parentId));
  for (const root of roots) {
    const world = operation.projection.transforms.get(root.id)!;
    setDraftTransform(operation.draft, identities.get(root.id)!, composeStructureTransforms(inverse, { ...world, x: world.x + command.delta.x, y: world.y + command.delta.y }));
  }
  const clonedRoots = roots.map((item) => identities.get(item.id)!);
  setDraftSiblings(operation.draft, command.parentId, siblings.slice(0, index).concat(clonedRoots, siblings.slice(index)));
  return { selection: clonedRoots, contentClones };
}
function validateIdentities(operation: StructureOperation, source: readonly StructureItem[], mapping: readonly { sourceId: string; targetId: string }[]) {
  const sourceIds = new Set(source.map((item) => item.id));
  const targets = new Set<string>(); const result = new Map<string, string>();
  for (const { sourceId, targetId } of mapping) {
    if (!sourceIds.has(sourceId) || result.has(sourceId) || targets.has(targetId)) throw new StructureError("invalid-reference", "Clone identities must form a one-to-one subtree mapping", [sourceId, targetId]);
    assertNewIdentity(operation.draft, targetId);
    result.set(sourceId, targetId); targets.add(targetId);
  }
  if (result.size !== source.length) throw new StructureError("invalid-reference", "Clone identities must include every descendant");
  return result;
}

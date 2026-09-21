import type { SpaceEditSnapshot } from "../canvas/contract/space-edit-snapshot";
import type { StructureNodeId, StructureNode } from "../model/structure/structure-contract";
import { ancestorsOf, effectiveOperationRoots } from "../model/structure/structure-index";
import { isStructureNodeEffectivelyLocked } from "../model/structure/structure-queries";
import { validateStructureSnapshot } from "../model/structure/structure-validation";

export type StructureCapability =
  | { enabled: true; roots: readonly StructureNode[] }
  | { enabled: false; reason: string };

export interface StructureParentTarget {
  id: StructureNodeId | null;
  title: string;
}

export function groupCapability(
  snapshot: SpaceEditSnapshot,
  selectedIds: readonly StructureNodeId[],
): StructureCapability {
  const index = validateStructureSnapshot(snapshot.structure, snapshot.layouts);
  const roots = effectiveOperationRoots(selectedIds, index);
  if (roots.length < 2) return { enabled: false, reason: "Select at least two nodes" };
  if (roots.some((structureNode) => structureNode.kind === "section"))
    return { enabled: false, reason: "Sections cannot be placed in a Group" };
  const parentId = roots[0]!.parentId;
  if (roots.some((structureNode) => structureNode.parentId !== parentId))
    return { enabled: false, reason: "Nodes must have the same direct parent" };
  if (roots.some((structureNode) => isStructureNodeEffectivelyLocked(structureNode.id, index)))
    return { enabled: false, reason: "Unlock the nodes first" };
  return { enabled: true, roots };
}

export function dissolveGroupCapability(
  snapshot: SpaceEditSnapshot,
  selectedIds: readonly StructureNodeId[],
): StructureCapability {
  const index = validateStructureSnapshot(snapshot.structure, snapshot.layouts);
  const roots = effectiveOperationRoots(selectedIds, index);
  if (!roots.length || roots.some((structureNode) => structureNode.kind !== "group"))
    return { enabled: false, reason: "Select one or more Groups" };
  if (roots.some((structureNode) => isStructureNodeEffectivelyLocked(structureNode.id, index)))
    return { enabled: false, reason: "Unlock the Groups first" };
  return { enabled: true, roots };
}

export function canRenameStructureNode(snapshot: SpaceEditSnapshot, selectedIds: readonly StructureNodeId[]) {
  if (selectedIds.length !== 1) return false;
  const index = validateStructureSnapshot(snapshot.structure, snapshot.layouts);
  return !isStructureNodeEffectivelyLocked(selectedIds[0]!, index);
}

export function structureParentTargets(
  snapshot: SpaceEditSnapshot,
  selectedIds: readonly StructureNodeId[],
  spaceRootTitle: string,
): StructureParentTarget[] {
  const index = validateStructureSnapshot(snapshot.structure, snapshot.layouts);
  const roots = effectiveOperationRoots(selectedIds, index);
  const parentId = roots[0]?.parentId;
  if (
    !roots.length ||
    roots.some(
      (structureNode) =>
        structureNode.kind === "section" ||
        structureNode.parentId !== parentId ||
        isStructureNodeEffectivelyLocked(structureNode.id, index),
    )
  )
    return [];
  const selected = new Set(roots.map(({ id }) => id));
  const containers = snapshot.structure.flatMap((structureNode) =>
    structureNode.kind !== "content" &&
    structureNode.id !== parentId &&
    !selected.has(structureNode.id) &&
    !isStructureNodeEffectivelyLocked(structureNode.id, index) &&
    !ancestorsOf(structureNode.id, index).some(({ id }) => selected.has(id))
      ? [{ id: structureNode.id, title: structureNode.title }]
      : [],
  );
  return parentId === null ? containers : [{ id: null, title: spaceRootTitle }, ...containers];
}

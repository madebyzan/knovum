import type { StructureNodeId } from "./structure-contract";
import type { StructureIndex } from "./structure-index";

export interface StructureOutlineRow {
  id: StructureNodeId;
  parentId: StructureNodeId | null;
  kind: "content" | "group" | "section";
  title: string;
  depth: number;
  locked: boolean;
  childCount: number;
}

/** Outline disclosure is navigation state; Section canvas collapse is deliberately ignored. */
export function projectStructureOutline(
  index: StructureIndex,
  contentTitles: ReadonlyMap<string, string>,
  expandedIds: ReadonlySet<StructureNodeId>,
): StructureOutlineRow[] {
  const rows: StructureOutlineRow[] = [];
  const roots = index.childrenByParent.get(null) ?? [];
  const pending = roots
    .toReversed()
    .map((structureNode) => ({ structureNode, depth: 0, inheritedLocked: false }));
  while (pending.length) {
    const { structureNode, depth, inheritedLocked } = pending.pop()!;
    const children = index.childrenByParent.get(structureNode.id) ?? [];
    const title =
      structureNode.kind === "content" ? contentTitles.get(structureNode.nodeId) : structureNode.title;
    if (title === undefined) throw new Error(`Content title missing for ${structureNode.id}`);
    const locked = inheritedLocked || structureNode.locked;
    rows.push({
      id: structureNode.id,
      parentId: structureNode.parentId,
      kind: structureNode.kind,
      title,
      depth,
      locked,
      childCount: children.length,
    });
    if (!expandedIds.has(structureNode.id)) continue;
    for (let childIndex = children.length - 1; childIndex >= 0; childIndex--)
      pending.push({ structureNode: children[childIndex]!, depth: depth + 1, inheritedLocked: locked });
  }
  return rows;
}

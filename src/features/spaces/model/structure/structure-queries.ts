import type { StructureNodeId, StructureLayout, StructureNode } from "./schema/structure";
import { ancestorsOf, type StructureIndex } from "./structure-index";

/** Resolve the complete content scope independently of folding, filtering and viewport mounting. */
export function contentNodeIdsWithin(rootIds: readonly StructureNodeId[], index: StructureIndex): string[] {
  const pending = rootIds.toReversed();
  const visited = new Set<StructureNodeId>();
  const content = new Set<string>();
  while (pending.length) {
    const id = pending.pop()!;
    if (visited.has(id)) continue;
    visited.add(id);
    const structureNode = index.structureById.get(id);
    if (!structureNode) throw new Error(`Structure structureNode not found: ${id}`);
    if (structureNode.kind === "content") {
      content.add(structureNode.nodeId);
      continue;
    }
    const children = index.childrenByParent.get(id) ?? [];
    for (let child = children.length - 1; child >= 0; child--) pending.push(children[child]!.id);
  }
  return [...content];
}

export function sectionAncestorOf(id: StructureNodeId, index: StructureIndex): StructureNode | undefined {
  return ancestorsOf(id, index).find((structureNode) => structureNode.kind === "section");
}

export function isStructureNodeEffectivelyLocked(id: StructureNodeId, index: StructureIndex): boolean {
  const structureNode = index.structureById.get(id);
  if (!structureNode) throw new Error(`Structure structureNode not found: ${id}`);
  return structureNode.locked || ancestorsOf(id, index).some(({ locked }) => locked);
}

/** Root-to-inner Group path required to reveal an structureNode in the editor. */
export function groupScopePathFor(id: StructureNodeId, index: StructureIndex): StructureNodeId[] {
  return ancestorsOf(id, index)
    .filter((structureNode) => structureNode.kind === "group")
    .map((structureNode) => structureNode.id);
}

/** Display endpoint only; knowledge relations retain their original NodeId endpoints. */
export function visibleRepresentativeForNode(
  nodeId: string,
  index: StructureIndex,
  layoutById: ReadonlyMap<StructureNodeId, StructureLayout>,
  temporarilyExpandedSectionIds: ReadonlySet<StructureNodeId> = new Set(),
): StructureNodeId | undefined {
  const structureNodeId = index.nodeIdToStructureNodeId.get(nodeId);
  if (!structureNodeId) return undefined;
  const section = sectionAncestorOf(structureNodeId, index);
  const sectionLayout = section ? layoutById.get(section.id) : undefined;
  if (
    section &&
    sectionLayout?.kind === "section" &&
    sectionLayout.collapsed &&
    !temporarilyExpandedSectionIds.has(section.id)
  )
    return section.id;
  return structureNodeId;
}

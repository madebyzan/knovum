import type { StructureNodeId, StructureNode } from "./schema/structure";

export interface StructureIndex {
  structureById: ReadonlyMap<StructureNodeId, StructureNode>;
  childrenByParent: ReadonlyMap<StructureNodeId | null, readonly StructureNode[]>;
  nodeIdToStructureNodeId: ReadonlyMap<string, StructureNodeId>;
}

export function createStructureIndex(structure: readonly StructureNode[]): StructureIndex {
  const structureById = new Map<StructureNodeId, StructureNode>();
  const childrenByParent = new Map<StructureNodeId | null, StructureNode[]>();
  const nodeIdToStructureNodeId = new Map<string, StructureNodeId>();
  for (const structureNode of structure) {
    if (structureById.has(structureNode.id)) throw new Error(`Duplicate structure node: ${structureNode.id}`);
    structureById.set(structureNode.id, structureNode);
    if (structureNode.kind === "content") {
      if (nodeIdToStructureNodeId.has(structureNode.nodeId))
        throw new Error(`Duplicate content reference: ${structureNode.nodeId}`);
      nodeIdToStructureNodeId.set(structureNode.nodeId, structureNode.id);
    }
    const siblings = childrenByParent.get(structureNode.parentId) ?? [];
    siblings.push(structureNode);
    childrenByParent.set(structureNode.parentId, siblings);
  }
  for (const siblings of childrenByParent.values()) siblings.sort((a, b) => a.order - b.order);
  return { structureById, childrenByParent, nodeIdToStructureNodeId };
}

/** Ancestors in root-to-parent order. The structure node itself is excluded. */
export function ancestorsOf(id: StructureNodeId, index: StructureIndex): StructureNode[] {
  const structureNode = index.structureById.get(id);
  if (!structureNode) throw new Error(`Structure node not found: ${id}`);
  const ancestors: StructureNode[] = [];
  const seen = new Set<StructureNodeId>([id]);
  let parentId = structureNode.parentId;
  while (parentId !== null) {
    if (seen.has(parentId)) throw new Error(`Structure cycle at: ${parentId}`);
    seen.add(parentId);
    const parent = index.structureById.get(parentId);
    if (!parent) throw new Error(`Structure parent not found: ${parentId}`);
    ancestors.push(parent);
    parentId = parent.parentId;
  }
  return ancestors.toReversed();
}

/** Keep selected structure nodes with no selected ancestor, preserving selection order. */
export function effectiveOperationRoots(
  ids: readonly StructureNodeId[],
  index: StructureIndex,
): StructureNode[] {
  const selected = new Set(ids);
  const roots: StructureNode[] = [];
  const added = new Set<StructureNodeId>();
  for (const id of ids) {
    const structureNode = index.structureById.get(id);
    if (!structureNode) throw new Error(`Structure node not found: ${id}`);
    if (added.has(id) || ancestorsOf(id, index).some((ancestor) => selected.has(ancestor.id))) continue;
    roots.push(structureNode);
    added.add(id);
  }
  return roots;
}

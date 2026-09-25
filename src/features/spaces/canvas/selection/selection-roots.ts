import { effectiveOperationRoots, type StructureIndex } from "../../model/structure/structure-index";

/** Interpret a transient canvas selection against the current structure snapshot. */
export function currentSelectionRoots(
  selectedNodeIds: Iterable<string>,
  selectedStructureNodeIds: Iterable<string>,
  index: StructureIndex,
) {
  const structureIds = [...selectedStructureNodeIds].filter((id) => index.structureById.has(id));
  for (const nodeId of selectedNodeIds) {
    const structureId = index.nodeIdToStructureNodeId.get(nodeId);
    if (structureId) structureIds.push(structureId);
  }
  return effectiveOperationRoots(structureIds, index);
}

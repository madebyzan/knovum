import type { StructureIndex } from "../../../model/structure/structure-index";
import { currentSelectionRoots } from "../../selection/selection-roots";

/** A selected container owns its descendants; they must not remain separate selected objects. */
export function normalizeCanvasSelection(
  nodeIds: Iterable<string>,
  structureNodeIds: Iterable<string>,
  index: StructureIndex,
) {
  const explicitNodes = new Set(nodeIds);
  const nodes = new Set<string>();
  const structures = new Set<string>();
  for (const root of currentSelectionRoots(explicitNodes, structureNodeIds, index)) {
    if (root.kind === "content" && explicitNodes.has(root.nodeId)) nodes.add(root.nodeId);
    else structures.add(root.id);
  }
  return { nodeIds: nodes, structureNodeIds: structures };
}

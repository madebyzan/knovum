import { canonicalRelationEndpoints, type SpaceRelation } from "../model/relations/relation";
import type { JsonCanvasEdgeV1 } from "./json-canvas-contract";

export function resolveImportedRelations(
  edges: readonly JsonCanvasEdgeV1[],
  importedNodes: readonly { sourceId: string; nodeId: string }[],
  spaceId: string,
): SpaceRelation[] {
  const nodeIds = new Map(importedNodes.map(({ sourceId, nodeId }) => [sourceId, nodeId]));
  const now = Date.now();
  return edges.map((edge) => {
    const [sourceId, targetId] = canonicalRelationEndpoints(
      importedNodeId(nodeIds, edge.fromNode),
      importedNodeId(nodeIds, edge.toNode),
      "related",
    );
    return {
      id: crypto.randomUUID(),
      spaceId,
      sourceId,
      targetId,
      kind: "related",
      rationale: edge.label ?? "",
      status: "active",
      actorKind: "human",
      createdAt: now,
      updatedAt: now,
    };
  });
}

function importedNodeId(nodeIds: ReadonlyMap<string, string>, sourceId: string) {
  const nodeId = nodeIds.get(sourceId);
  if (!nodeId) throw new Error(`Imported edge references unavailable node ${sourceId}`);
  return nodeId;
}

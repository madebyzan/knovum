import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";

const NON_SEMANTIC_KINDS = new Set<SpaceNodePayload["kind"]>(["credential", "section", "portal", "text"]);

export function isSpaceSemanticNodeData(data: SpaceNodePayload) {
  return !NON_SEMANTIC_KINDS.has(data.kind);
}

export function isSpaceSemanticNode(node: SpaceCanvasNode) {
  return isSpaceSemanticNodeData(node.data);
}

import type { CanvasPackageCodec } from "../package/canvas-state-codec";
import type { PackagedCanvasState } from "../package/core-schema";
import { parseCanvasDocument } from "./contract/canvas-document";

export const canvasPackageCodec: CanvasPackageCodec = {
  validate: (source) => void parseCanvasDocument(source.document),
  remap: remapCanvasState,
};

function remapCanvasState(
  source: PackagedCanvasState,
  context: Parameters<CanvasPackageCodec["remap"]>[1],
): PackagedCanvasState {
  const document = parseCanvasDocument(source.document);
  return {
    document: parseCanvasDocument({
      ...document,
      nodes: document.nodes.map((node) =>
        Object.assign({}, node, { nodeId: mappedNodeId(context.nodeIds, node.nodeId) }),
      ),
    }),
  };
}

function mappedNodeId(nodeIds: ReadonlyMap<string, string>, sourceId: string) {
  const targetId = nodeIds.get(sourceId);
  if (!targetId) throw new Error(`Canvas node ${sourceId} is missing from the package node map`);
  return targetId;
}

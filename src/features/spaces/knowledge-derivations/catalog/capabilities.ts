import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { GroundedDocumentSource } from "./grounded-document";

export interface SelectionCapabilities {
  sources: readonly GroundedDocumentSource[] | undefined;
  imageCount: number;
}

export function selectionCapabilities(
  nodes: readonly SpaceCanvasNode[],
  sources: readonly GroundedDocumentSource[] | undefined,
): SelectionCapabilities {
  return {
    sources,
    imageCount: nodes.filter(({ data }) => data.kind === "image" && data.sourceRevision).length,
  };
}

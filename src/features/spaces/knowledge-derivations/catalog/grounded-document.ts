import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { hasDerivableKnowledge } from "./source-capability";

export interface GroundedDocumentSource {
  nodeId: string;
  title: string;
  role: "primary" | "source";
  sourceRevision: string;
}

export function groundedDocumentSources(
  nodes: readonly SpaceCanvasNode[],
): GroundedDocumentSource[] | undefined {
  if (nodes.length === 0 || nodes.length > 32) return;
  const sources: GroundedDocumentSource[] = [];
  for (const [index, node] of nodes.entries()) {
    const revision = node.data.knowledgeRevision ?? node.data.sourceRevision;
    if (!hasDerivableKnowledge(node.data) || !revision) return;
    sources.push({
      nodeId: node.id,
      title: node.data.title,
      role: index === 0 ? "primary" : "source",
      sourceRevision: revision,
    });
  }
  return sources;
}

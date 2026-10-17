import type { RevisionBasis } from "../../contracts/generated/RevisionBasis";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { hasDerivableKnowledge } from "./source-capability";

/**
 * A selected resource, pinned to the revision the user is looking at. Text is
 * pinned on its indexed knowledge; an image is pinned on its pixels, so a
 * picture nothing has analysed yet is still something to create from.
 */
export interface CreationSource {
  nodeId: string;
  title: string;
  sourceRevision: string;
  revisionBasis: RevisionBasis;
}

/** The canvas pins at most this many resources per Run. */
export const MAX_CREATION_SOURCES = 32;

export function creationSources(nodes: readonly SpaceCanvasNode[]): CreationSource[] {
  return nodes.flatMap((node) => {
    const pinned = pinnedRevision(node);
    return pinned ? [{ nodeId: node.id, title: node.data.title, ...pinned }] : [];
  });
}

function pinnedRevision(
  node: SpaceCanvasNode,
): { sourceRevision: string; revisionBasis: RevisionBasis } | undefined {
  const { data } = node;
  if (data.kind === "image") {
    // Pixels are readable on their own: no analysis has to finish first.
    return data.sourceRevision && !data.processingState
      ? { sourceRevision: data.sourceRevision, revisionBasis: "asset" }
      : undefined;
  }
  return data.knowledgeRevision && hasDerivableKnowledge(data)
    ? { sourceRevision: data.knowledgeRevision, revisionBasis: "knowledge" }
    : undefined;
}

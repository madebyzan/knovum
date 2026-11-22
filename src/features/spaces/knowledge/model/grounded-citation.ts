import type { KnowledgePeekTarget } from "./knowledge-peek-target";
import type { GroundedCitation } from "./schema/grounded-citation";

export function findGroundedCitation(
  citations: readonly GroundedCitation[] | undefined,
  targetIndex: number,
): GroundedCitation | undefined {
  if (!citations?.length) return undefined;
  return citations.find((citation) => citation.index === targetIndex);
}

/** Projects a live Grounded citation into the Source Reader peek target shared by all surfaces. */
export function groundedCitationPeekTarget(
  citation: GroundedCitation | undefined,
): KnowledgePeekTarget | undefined {
  if (citation?.state !== "current") return undefined;
  return {
    nodeId: citation.nodeId,
    nodeTitle: citation.nodeTitle,
    chunkId: citation.chunkId,
    locator: citation.locator,
    text: citation.quote,
  };
}

import type { GroundedCitation } from "./schema/grounded-citation";

export function findGroundedCitation(
  citations: readonly GroundedCitation[] | undefined,
  targetIndex: number,
): GroundedCitation | undefined {
  if (!citations?.length) return undefined;
  return citations.find((citation) => citation.index === targetIndex);
}

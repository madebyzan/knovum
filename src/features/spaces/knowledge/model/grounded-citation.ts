import { z } from "zod";

import { knowledgeLocatorSchema } from "./schema";

const groundedCitationSnapshotSchema = {
  index: z.number().int().positive(),
  sourceSpaceTitle: z.string().min(1).max(512).optional(),
  nodeTitle: z.string().min(1).max(512),
  sourceRevision: z.string().min(1),
  locator: knowledgeLocatorSchema,
  quote: z.string().min(1).max(64_000),
} as const;

export const groundedCitationSchema = z.discriminatedUnion("state", [
  z
    .object({
      state: z.literal("current"),
      chunkId: z.string().min(1),
      nodeId: z.string().min(1),
      ...groundedCitationSnapshotSchema,
    })
    .strict(),
  z.object({ state: z.literal("detached"), ...groundedCitationSnapshotSchema }).strict(),
]);

export type GroundedCitation = z.infer<typeof groundedCitationSchema>;

export function findGroundedCitation(
  citations: readonly GroundedCitation[] | undefined,
  targetIndex: number,
): GroundedCitation | undefined {
  if (!citations?.length) return undefined;
  return citations.find((citation) => citation.index === targetIndex);
}

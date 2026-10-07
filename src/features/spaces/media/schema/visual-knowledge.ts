import { z } from "zod";

import { executionSourceSchema } from "../../../../foundation/ai/schema/models";
import { visualKnowledgeLimits } from "../../contracts/generated/contract-values";

function nonEmptyText(maxUnits: number) {
  return z
    .string()
    .refine((value) => value.trim().length > 0, "Must be non-empty")
    .refine((value) => value.length <= maxUnits, `Must not exceed ${maxUnits} UTF-16 code units`);
}

export const spaceVisualKnowledgeSchema = z
  .object({
    sourceRevision: nonEmptyText(128),
    description: nonEmptyText(visualKnowledgeLimits.description),
    detectedText: nonEmptyText(visualKnowledgeLimits.detectedText).optional(),
    tags: z.array(nonEmptyText(visualKnowledgeLimits.tag)).max(visualKnowledgeLimits.tags),
    generatedAt: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
    executionSource: executionSourceSchema,
  })
  .strict();

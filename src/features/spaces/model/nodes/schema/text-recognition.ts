import { z } from "zod";

import { textRecognitionLimits } from "../../../contracts/generated/contract-values";

export const textRecognitionSchema = z
  .object({
    sourceRevision: z.string().min(1).max(128),
    modelSpecId: z.string().min(1).max(160),
    generatedAt: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
    text: z.string().min(1).max(textRecognitionLimits.text).optional(),
  })
  .strict();

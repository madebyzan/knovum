import { z } from "zod";

/** That a node's pictures were read, and from which input; the text lives in the node's passages. */
export const textRecognitionSchema = z
  .object({
    sourceRevision: z.string().min(1).max(128),
    modelSpecId: z.string().min(1).max(160),
    generatedAt: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
  })
  .strict();

import { z } from "zod";

export const spaceMediaAnalysisSchema = z
  .object({
    schemaVersion: z.literal(1),
    sourceRevision: z.string().trim().min(1).max(128),
    durationMs: z.number().int().positive().max(Number.MAX_SAFE_INTEGER),
    waveform: z.array(z.number().finite().min(0).max(1)).min(1).max(1_200),
  })
  .strict();

import { z } from "zod";

export const objectFieldSchema = z
  .object({
    key: z.string().min(1).max(128),
    label: z.string().min(1).max(128),
    kind: z.enum(["text", "number", "boolean", "date", "node-ref", "enum"]),
    required: z.boolean().optional(),
    options: z.array(z.string().trim().min(1).max(128)).min(1).max(64).optional(),
  })
  .strict();

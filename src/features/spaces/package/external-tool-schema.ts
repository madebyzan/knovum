import { z } from "zod";

import { packageTimeSchema } from "./core-schema";

export const packageExternalToolBindingSchema = z
  .object({
    id: z.string().min(1),
    name: z.string().min(1),
    endpoint: z.string().url(),
    serverIdentity: z.record(z.string(), z.unknown()),
    toolName: z.string().min(1),
    inputSchema: z.record(z.string(), z.unknown()),
    outputSchema: z.record(z.string(), z.unknown()).nullable(),
    schemaFingerprint: z.string().length(64),
    effect: z.enum(["read_only", "additive", "destructive"]),
    reviewedAt: packageTimeSchema,
    createdAt: packageTimeSchema,
    updatedAt: packageTimeSchema,
  })
  .strict();

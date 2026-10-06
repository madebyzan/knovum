import { z } from "zod";
export const externalContentSourceSchema = z
  .object({
    type: z.literal("mcp_resource"),
    connectionId: z.string().min(1).max(128),
    connectionRevision: z.string().min(1).max(128),
    catalogDigest: z.string().regex(/^[a-f0-9]{64}$/i),
    uri: z.string().min(1).max(8192),
    contentDigest: z.string().regex(/^[a-f0-9]{64}$/i),
    capturedAt: z.number().int().nonnegative().safe(),
  })
  .strict();

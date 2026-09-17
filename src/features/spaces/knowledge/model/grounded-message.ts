import { z } from "zod";

import { groundedCitationSchema } from "./grounded-citation";
import { knowledgeTimestampSchema } from "./schema";

export const groundedMessageSchema = z
  .object({
    id: z.string().min(1),
    threadId: z.string().min(1),
    role: z.enum(["user", "assistant"]),
    content: z.string().max(1_000_000),
    citations: z.array(groundedCitationSchema).max(1_000),
    provider: z.string().max(512).optional(),
    model: z.string().max(512).optional(),
    createdAt: knowledgeTimestampSchema,
  })
  .strict();

export type GroundedMessage = z.infer<typeof groundedMessageSchema>;

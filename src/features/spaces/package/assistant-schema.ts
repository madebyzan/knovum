import { z } from "zod";

import { packageIdSchema, packageTimeSchema } from "./core-schema";

export const packageAssistantConversationSchema = z
  .object({
    id: packageIdSchema,
    title: z.string().min(1).max(512),
    mode: z.enum(["ask", "plan", "execute"]),
    archivedAt: packageTimeSchema.nullable(),
    createdAt: packageTimeSchema,
    updatedAt: packageTimeSchema,
  })
  .strict();

export const packageAssistantMessageSchema = z
  .object({
    id: packageIdSchema,
    conversationId: packageIdSchema,
    branchId: packageIdSchema,
    parentMessageId: packageIdSchema.nullable(),
    role: z.enum(["user", "assistant", "tool"]),
    content: z.object({ parts: z.array(z.unknown()).max(10_000) }).strict(),
    createdAt: packageTimeSchema,
  })
  .strict();

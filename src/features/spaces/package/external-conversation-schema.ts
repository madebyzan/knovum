import { z } from "zod";

import { agentKindSchema } from "../../../foundation/ai/agent-models";
import { packageIdSchema, packageTimeSchema } from "./core-schema";

export const packageExternalAgentConversationSchema = z
  .object({
    id: packageIdSchema,
    runId: packageIdSchema,
    runtimeKind: agentKindSchema,
    runtimeSessionId: z.string().min(1).nullable(),
    sessionCapabilities: z.unknown().nullable(),
    title: z.string().min(1).max(160),
    state: z.enum(["connecting", "connected", "disconnected", "closed"]),
    error: z.string().nullable(),
    createdAt: packageTimeSchema,
    updatedAt: packageTimeSchema,
  })
  .strict();

export const packageExternalAgentTurnSchema = z
  .object({
    id: packageIdSchema,
    conversationId: packageIdSchema,
    state: z.enum(["running", "completed", "failed", "cancelled"]),
    error: z.string().nullable(),
    createdAt: packageTimeSchema,
    completedAt: packageTimeSchema.nullable(),
  })
  .strict();

export const packageExternalAgentMessageSchema = z
  .object({
    id: packageIdSchema,
    conversationId: packageIdSchema,
    turnId: packageIdSchema,
    role: z.enum(["user", "assistant"]),
    content: z.string(),
    createdAt: packageTimeSchema,
  })
  .strict();

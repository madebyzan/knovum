import { z } from "zod";

import { packageTimeSchema } from "./core-schema";

export const packageAutomationTriggerSchema = z
  .object({
    id: z.string().min(1),
    name: z.string().min(1),
    workflowId: z.string().min(1),
    workflowVersion: z.number().int().positive(),
    source: z.discriminatedUnion("kind", [
      z
        .object({
          kind: z.literal("cron"),
          schedule: z.string(),
          timezone: z.string(),
          misfirePolicy: z.enum(["skip", "latest", "catch_up"]),
          misfireWindowSeconds: z.number().int(),
          catchUpLimit: z.number().int(),
        })
        .strict(),
      z
        .object({
          kind: z.literal("event"),
          eventKind: z.string(),
          filterExpression: z.string().nullable(),
          causalDepthLimit: z.number().int(),
          rateLimitCount: z.number().int(),
          rateLimitWindowSeconds: z.number().int(),
        })
        .strict(),
    ]),
    input: z.unknown(),
    overlapPolicy: z.enum(["skip", "queue", "parallel", "cancel_previous"]),
    createdAt: packageTimeSchema,
    updatedAt: packageTimeSchema,
  })
  .strict();

export const packageAutomationOccurrenceSchema = z
  .object({
    id: z.string().min(1),
    triggerId: z.string().min(1),
    scheduledAt: packageTimeSchema,
    occurrenceKey: z.string().min(1),
    runId: z.string().nullable(),
    state: z.string().min(1),
    detail: z.unknown(),
    createdAt: packageTimeSchema,
  })
  .strict();

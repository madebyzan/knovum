import { z } from "zod";

import type { WorkflowDefinition } from "../contracts/generated/WorkflowDefinition";

const workflowDefinitionSchema = z
  .object({
    schemaVersion: z.literal(1),
    title: z.string().min(1).max(512),
    nodes: z.array(z.record(z.string(), z.unknown())).min(1).max(200),
    edges: z.array(z.record(z.string(), z.unknown())).max(20_000),
    layout: z.record(z.string(), z.object({ x: z.number(), y: z.number() }).strict()),
    failurePolicy: z.enum(["stop_run", "continue_independent", "collect_errors"]),
    limits: z
      .object({
        maxExpandedSteps: z.number().int().positive().max(10_000),
        maxParallelism: z.number().int().positive().max(64),
        maxDurationSeconds: z.number().int().positive().max(604_800),
        maxTotalTokens: z.number().int().positive().max(10_000_000),
        maxStepAttempts: z.number().int().positive().max(100_000),
        maxInputBytes: z.number().int().positive().max(1_073_741_824),
        maxOutputBytes: z.number().int().positive().max(1_073_741_824),
      })
      .strict(),
  })
  .strict()
  .transform((value) => value as WorkflowDefinition);

export const workflowDraftSchema = z
  .object({
    id: z.string().min(1).max(128),
    revision: z.number().int().nonnegative(),
    definition: workflowDefinitionSchema,
    createdAt: z.number().int(),
    updatedAt: z.number().int(),
  })
  .strict();

export const workflowVersionSchema = z
  .object({
    workflowId: z.string().min(1).max(128),
    version: z.number().int().positive(),
    definition: workflowDefinitionSchema,
    publishedAt: z.number().int(),
  })
  .strict();

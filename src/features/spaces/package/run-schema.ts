import { z } from "zod";

import { packageTimeSchema } from "./core-schema";

const jsonValue = z.unknown();

export const packageRunSchema = z
  .object({
    id: z.string().min(1),
    parentRunId: z.string().min(1).nullable(),
    retryOfRunId: z.string().min(1).nullable(),
    forkOfRunId: z.string().min(1).nullable(),
    principalKind: z.enum(["user", "assistant", "external_agent"]),
    principalId: z.string().min(1),
    kind: z.enum(["action", "workflow", "assistant", "external_agent"]),
    state: z.enum(["succeeded", "completed_with_errors", "failed", "cancelled"]),
    stateVersion: z.number().int().nonnegative(),
    policySnapshot: jsonValue,
    inputSnapshot: jsonValue,
    budget: jsonValue,
    inputTokens: z.number().int().nonnegative(),
    outputTokens: z.number().int().nonnegative(),
    totalTokens: z.number().int().nonnegative(),
    stepAttempts: z.number().int().nonnegative(),
    expandedSteps: z.number().int().nonnegative(),
    outputBytes: z.number().int().nonnegative(),
    createdAt: packageTimeSchema,
    updatedAt: packageTimeSchema,
    completedAt: packageTimeSchema,
  })
  .strict();

export const packageRunStepSchema = z
  .object({
    id: z.string().min(1),
    runId: z.string().min(1),
    definitionNodeId: z.string().min(1),
    instancePath: z.string().min(1),
    nodeKind: z.string().min(1),
    dependsOn: z.array(z.string()),
    configuration: jsonValue,
    state: z.string().min(1),
    stateVersion: z.number().int().nonnegative(),
    resolvedInput: jsonValue,
    output: jsonValue.nullable(),
    createdAt: packageTimeSchema,
    updatedAt: packageTimeSchema,
  })
  .strict();

export const packageRunAttemptSchema = z
  .object({
    id: z.string().min(1),
    runId: z.string().min(1),
    stepInstanceId: z.string().min(1),
    attemptNumber: z.number().int().positive(),
    state: z.string().min(1),
    requestDigest: z.string().length(64),
    idempotencyKey: z.string().min(1),
    externalTaskId: z.string().nullable(),
    checkpoint: jsonValue.nullable(),
    result: jsonValue.nullable(),
    errorCode: z.string().nullable(),
    startedAt: packageTimeSchema,
    finishedAt: packageTimeSchema.nullable(),
  })
  .strict();

export const packageRunEventSchema = z
  .object({
    runId: z.string().min(1),
    stepInstanceId: z.string().nullable(),
    attemptId: z.string().nullable(),
    kind: z.enum([
      "assistant_paused_checkpoint",
      "attempt_claimed",
      "external_agent_message",
      "external_agent_permission_requested",
      "external_agent_tool_started",
      "external_agent_tool_updated",
      "external_outcome_reconciled",
      "run_budget_exhausted",
      "run_queued",
      "run_retry_created",
      "step_failed",
      "step_retry_scheduled",
    ]),
    payload: jsonValue,
    createdAt: packageTimeSchema,
  })
  .strict();

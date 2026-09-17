import { uniqBy } from "es-toolkit";
import { z } from "zod";

import { packageTimeSchema } from "./core-schema";

const workflowStepSchema = z
  .object({
    id: z.string().min(1).max(128),
    action: z.string().min(1).max(128),
    status: z.enum(["pending", "succeeded", "failed", "cancelled"]),
    outputNodeIds: z.array(z.string().min(1)).max(1_000),
    error: z.string().max(16_000),
    createdAt: packageTimeSchema,
    updatedAt: packageTimeSchema,
    completedAt: packageTimeSchema.nullable(),
  })
  .strict()
  .superRefine((step, context) => {
    const valid =
      step.status === "pending"
        ? step.error === "" && step.completedAt === null
        : step.status === "succeeded" || step.status === "cancelled"
          ? step.error === "" && step.completedAt !== null
          : step.error.trim().length > 0 && step.completedAt !== null;
    if (!valid) context.addIssue({ code: "custom", message: "Workflow step lifecycle is invalid" });
  });

export const workflowRunSchema = z
  .object({
    id: z.string().min(1),
    workflowId: z.string().min(1).max(128),
    workflowVersion: z.number().int().positive(),
    status: z.enum(["completed", "failed", "cancelled"]),
    error: z.string().max(16_000),
    createdAt: packageTimeSchema,
    updatedAt: packageTimeSchema,
    completedAt: packageTimeSchema,
    steps: z.array(workflowStepSchema).min(1).max(1_000),
  })
  .strict()
  .superRefine((run, context) => {
    const failed = run.steps.some(({ status }) => status === "failed");
    const cancelled = run.steps.some(({ status }) => status === "cancelled");
    const completed = run.steps.every(({ status }) => status === "succeeded");
    const valid =
      run.status === "failed"
        ? failed && run.error.trim().length > 0
        : run.status === "cancelled"
          ? cancelled && !failed && run.error === ""
          : completed && run.error === "";
    if (!valid) context.addIssue({ code: "custom", message: "Workflow run lifecycle is invalid" });
    if (uniqBy(run.steps, ({ id }) => id).length !== run.steps.length) {
      context.addIssue({ code: "custom", message: "Workflow run contains duplicate step IDs" });
    }
  });

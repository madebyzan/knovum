import { z } from "zod";

import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import type { StepSummary } from "../contracts/generated/StepSummary";
import { getNode } from "../nodes/node-operations";

/** `StepSummary.output` is a generated `JsonValue`, so the review checkpoint is an untrusted shape. */
const reviewCheckpointSchema = z.object({
  nodeId: z.string().min(1),
  sourceRevision: z.string().min(1),
});

function reviewCheckpoint(step: StepSummary | undefined) {
  const checkpoint = reviewCheckpointSchema.safeParse(step?.output);
  if (!step || !checkpoint.success) throw new Error("workflow-review-unavailable");
  return { step, ...checkpoint.data };
}

export async function loadWorkflowReview(spaceId: string, runId: string) {
  const steps = await invoke<StepSummary[]>("plugin:spaces|list_run_steps", { runId });
  const { step, nodeId, sourceRevision } = reviewCheckpoint(
    steps.find((item) => item.state === "waiting" && item.waitReason === "content_review"),
  );
  const node = await getNode(spaceId, nodeId);
  if (node.data.kind !== "markdown" && node.data.kind !== "subtitle")
    throw new Error("workflow-review-unavailable");
  const revision = node.data.knowledgeRevision ?? node.data.sourceRevision;
  if (!revision) throw new Error("workflow-review-unavailable");
  return {
    step,
    content: node.data,
    nodeId,
    revision,
    edited: revision !== sourceRevision,
  };
}

export function decideWorkflowReview(input: {
  stepId: string;
  expectedVersion: number;
  reviewedRevision: string;
  approved: boolean;
}) {
  return invoke<void>("plugin:spaces|decide_run_step_review", input);
}

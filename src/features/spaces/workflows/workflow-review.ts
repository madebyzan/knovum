import { invoke } from "@tauri-apps/api/core";

import type { StepSummary } from "../contracts/generated/RunStep";
import { getNode } from "../nodes/node-repository";

function reviewCheckpoint(step: StepSummary | undefined) {
  const output = step?.output;
  if (!step || typeof output !== "object" || output === null) throw new Error("workflow-review-unavailable");
  const checkpoint = output as Record<string, unknown>;
  if (typeof checkpoint.nodeId !== "string" || typeof checkpoint.sourceRevision !== "string")
    throw new Error("workflow-review-unavailable");
  return { step, nodeId: checkpoint.nodeId, sourceRevision: checkpoint.sourceRevision };
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

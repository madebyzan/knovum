import { invoke } from "@tauri-apps/api/core";

import type { ApprovalSummary as ActionApproval } from "../contracts/generated/ActionApproval";
import type { BatchItemSummary } from "../contracts/generated/RunBatchItem";
import type { RunEvent } from "../contracts/generated/RunEvent";
import type { StepSummary } from "../contracts/generated/RunStep";
import type { RunSummary } from "../contracts/generated/RunSummary";

export type { ActionApproval, RunEvent, RunSummary };
export type RunStep = StepSummary;
export type RunBatchItem = BatchItemSummary;

export function submitRunStepInput(stepId: string, expectedVersion: number, value: unknown): Promise<void> {
  return invoke("plugin:spaces|submit_run_step_input", { stepId, expectedVersion, value });
}

export function decideRunStepReview(
  stepId: string,
  expectedVersion: number,
  approved: boolean,
): Promise<void> {
  return invoke("plugin:spaces|decide_run_step_review", { stepId, expectedVersion, approved });
}

export function reconcileRunStep(
  stepId: string,
  expectedVersion: number,
  decision: "retry" | "fail",
): Promise<void> {
  return invoke("plugin:spaces|reconcile_run_step", { stepId, expectedVersion, decision });
}

export function continueRunStep(stepId: string, expectedVersion: number): Promise<void> {
  return invoke("plugin:spaces|continue_run_step", { stepId, expectedVersion });
}

export function listRuns(spaceId: string): Promise<RunSummary[]> {
  return invoke("plugin:spaces|list_runs", { spaceId });
}

export function listRunSteps(runId: string): Promise<RunStep[]> {
  return invoke("plugin:spaces|list_run_steps", { runId });
}

export function listRunBatchItems(runId: string): Promise<RunBatchItem[]> {
  return invoke("plugin:spaces|list_run_batch_items", { runId });
}

export function listRunEvents(runId: string, afterSequence = 0): Promise<RunEvent[]> {
  return invoke("plugin:spaces|list_run_events", { runId, afterSequence });
}

export function getRunDiagnostics(runId: string): Promise<unknown> {
  return invoke("plugin:spaces|get_run_diagnostics", { runId });
}

export function controlRun(control: "cancel" | "pause" | "resume", runId: string): Promise<void> {
  return invoke(`plugin:spaces|${control}_run`, { runId });
}

export function retryRun(runId: string, stepIds: string[] = []): Promise<RunSummary> {
  return invoke("plugin:spaces|retry_run", { runId, stepIds });
}

export function listPendingActionApprovals(spaceId: string): Promise<ActionApproval[]> {
  return invoke("plugin:spaces|list_pending_action_approvals", { spaceId });
}

export function decideActionApproval(approvalId: string, approve: boolean): Promise<void> {
  return invoke("plugin:spaces|decide_action_approval", {
    approvalId,
    decision: approve ? "approve" : "reject",
  });
}

export function decideExternalAgentPermission(
  sessionId: string,
  requestId: string,
  optionId: string | null,
): Promise<void> {
  return invoke("plugin:spaces|decide_external_agent_permission", { sessionId, requestId, optionId });
}

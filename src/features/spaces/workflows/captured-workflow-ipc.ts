import type { CapturedRunOutput } from "../contracts/generated/CapturedRunOutput";
import type { CapturedWorkflowLibrary } from "../contracts/generated/CapturedWorkflowLibrary";
import type { ReviseCapturedWorkflowRequest } from "../contracts/generated/ReviseCapturedWorkflowRequest";
import type { StartCapturedWorkflowRequest } from "../contracts/generated/StartCapturedWorkflowRequest";

export type { CapturedInput } from "../contracts/generated/CapturedInput";

export type { CaptureIssueReason as WorkflowCaptureIssueReason } from "../contracts/generated/CaptureIssueReason";

export type { CaptureCandidate as WorkflowCaptureCandidate } from "../contracts/generated/CaptureCandidate";

export type { CapturedStep } from "../contracts/generated/CapturedStep";
import type { CapturePreview as WorkflowCapturePreview } from "../contracts/generated/CapturePreview";
export type { CapturePreview as WorkflowCapturePreview } from "../contracts/generated/CapturePreview";
import type { SaveCapturedWorkflowRequest } from "../contracts/generated/SaveCapturedWorkflowRequest";
export type { SaveCapturedWorkflowRequest } from "../contracts/generated/SaveCapturedWorkflowRequest";
import type { SavedCapturedWorkflow } from "../contracts/generated/SavedCapturedWorkflow";
export type { SavedCapturedWorkflow } from "../contracts/generated/SavedCapturedWorkflow";
import type { CapturedWorkflowRun } from "../contracts/generated/CapturedWorkflowRun";
export type { CapturedWorkflowRun } from "../contracts/generated/CapturedWorkflowRun";
import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import type { RunObservation } from "../contracts/generated/RunObservation";

export function previewWorkflowCapture(
  spaceId: string,
  selectedNodeIds: string[],
): Promise<WorkflowCapturePreview> {
  return invoke("plugin:spaces|preview_workflow_capture", { spaceId, selectedNodeIds });
}

export function saveCapturedWorkflow(request: SaveCapturedWorkflowRequest): Promise<SavedCapturedWorkflow> {
  return invoke("plugin:spaces|save_captured_workflow", { request });
}

export function listCapturedWorkflows(): Promise<CapturedWorkflowLibrary> {
  return invoke("plugin:spaces|list_captured_workflows");
}

export function renameCapturedWorkflow(
  workflowId: string,
  expectedName: string,
  name: string,
): Promise<SavedCapturedWorkflow> {
  return invoke("plugin:spaces|rename_captured_workflow", { workflowId, expectedName, name });
}

export function reviseCapturedWorkflow(
  request: ReviseCapturedWorkflowRequest,
): Promise<SavedCapturedWorkflow> {
  return invoke("plugin:spaces|revise_captured_workflow", { request });
}

export function deleteCapturedWorkflow(workflowId: string, expectedName: string): Promise<void> {
  return invoke("plugin:spaces|delete_captured_workflow", { workflowId, expectedName });
}

export function startCapturedWorkflow(request: StartCapturedWorkflowRequest): Promise<RunObservation> {
  return invoke("plugin:spaces|start_captured_workflow", { request });
}

export function listCapturedRunOutputs(runId: string): Promise<CapturedRunOutput[]> {
  return invoke("plugin:spaces|list_captured_run_outputs", { runId });
}

export function listCapturedWorkflowRuns(spaceId: string): Promise<CapturedWorkflowRun[]> {
  return invoke("plugin:spaces|list_captured_workflow_runs", { spaceId });
}

export function retryCapturedWorkflowRun(runId: string): Promise<RunObservation> {
  return invoke("plugin:spaces|retry_run", { runId, stepIds: [] });
}

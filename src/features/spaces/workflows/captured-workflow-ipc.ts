import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { CapturedRunOutput } from "../contracts/generated/CapturedRunOutput";
import type { CapturedWorkflowLibrary } from "../contracts/generated/CapturedWorkflowLibrary";
import type { CapturedWorkflowRun } from "../contracts/generated/CapturedWorkflowRun";
import type { CapturePreview } from "../contracts/generated/CapturePreview";
import type { ReviseCapturedWorkflowRequest } from "../contracts/generated/ReviseCapturedWorkflowRequest";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { SaveCapturedWorkflowRequest } from "../contracts/generated/SaveCapturedWorkflowRequest";
import type { SavedCapturedWorkflow } from "../contracts/generated/SavedCapturedWorkflow";
import type { StartCapturedWorkflowRequest } from "../contracts/generated/StartCapturedWorkflowRequest";

export function previewWorkflowCapture(spaceId: string, selectedNodeIds: string[]): Promise<CapturePreview> {
  return invokeCommand("plugin:spaces|preview_workflow_capture", { spaceId, selectedNodeIds });
}

export function saveCapturedWorkflow(request: SaveCapturedWorkflowRequest): Promise<SavedCapturedWorkflow> {
  return invokeCommand("plugin:spaces|save_captured_workflow", { request });
}

export function listCapturedWorkflows(): Promise<CapturedWorkflowLibrary> {
  return invokeCommand("plugin:spaces|list_captured_workflows");
}

export function renameCapturedWorkflow(
  workflowId: string,
  expectedName: string,
  name: string,
): Promise<SavedCapturedWorkflow> {
  return invokeCommand("plugin:spaces|rename_captured_workflow", { workflowId, expectedName, name });
}

export function reviseCapturedWorkflow(
  request: ReviseCapturedWorkflowRequest,
): Promise<SavedCapturedWorkflow> {
  return invokeCommand("plugin:spaces|revise_captured_workflow", { request });
}

export function deleteCapturedWorkflow(workflowId: string, expectedName: string): Promise<void> {
  return invokeCommand("plugin:spaces|delete_captured_workflow", { workflowId, expectedName });
}

export function startCapturedWorkflow(request: StartCapturedWorkflowRequest): Promise<RunObservation> {
  return invokeCommand("plugin:spaces|start_captured_workflow", { request });
}

export function listCapturedRunOutputs(runId: string): Promise<CapturedRunOutput[]> {
  return invokeCommand("plugin:spaces|list_captured_run_outputs", { runId });
}

export function listCapturedWorkflowRuns(spaceId: string): Promise<CapturedWorkflowRun[]> {
  return invokeCommand("plugin:spaces|list_captured_workflow_runs", { spaceId });
}

export function retryCapturedWorkflowRun(runId: string): Promise<RunObservation> {
  return invokeCommand("plugin:spaces|retry_run", { runId, stepIds: [] });
}

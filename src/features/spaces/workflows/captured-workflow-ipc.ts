import { invoke } from "@tauri-apps/api/core";

import type { RunSummary } from "../contracts/generated/RunSummary";

export interface CapturedInput {
  nodeId: string;
  kind: string;
  role: string;
  position: number;
  sourceRevision: string;
  revisionBasis: string;
}

export type WorkflowCaptureIssueReason =
  | "resource-unavailable"
  | "derivation-incomplete"
  | "execution-spec-unavailable"
  | "derivation-inputs-unavailable"
  | "source-unavailable"
  | "source-mismatch"
  | "source-identity-unavailable"
  | "parameter-instance-reference"
  | "candidate-binding-unavailable"
  | "no-derived-output-selected";

export interface WorkflowCaptureCandidate {
  inputSlots: Array<{
    id: string;
    kind: string;
    revisionBases: string[];
    minItems: number;
    maxItems: number;
    ordered: boolean;
  }>;
  steps: Array<{
    id: string;
    actionId: string;
    actionVersion: number;
    outputKind: string;
    executionSpec: CapturedStep["executionSpec"];
    inputs: Array<
      {
        role: string;
        position: number;
        revisionBasis: string;
      } & (
        | { sourceType: "inputSlot"; inputSlotId: string }
        | { sourceType: "inputSequence"; inputSlotId: string }
        | { sourceType: "stepOutput"; stepId: string }
      )
    >;
  }>;
  outputSlots: Array<{ id: string; name: string; kind: string; stepId: string }>;
}

export interface CapturedStep {
  derivationId: string;
  outputNodeId: string;
  outputKind: string;
  actionId: string;
  actionVersion: number;
  executionSpec: {
    parameters: Record<string, unknown>;
    target: { type: "model"; connectionId: string; modelId: string; temperature: number } | null;
  };
  inputs: CapturedInput[];
  selected: boolean;
}

export interface WorkflowCapturePreview {
  steps: CapturedStep[];
  inputs: Array<{
    nodeId: string;
    kind: string;
    selected: boolean;
    uses: Array<{
      derivationId: string;
      position: number;
      role: string;
      sourceRevision: string;
      revisionBasis: string;
    }>;
  }>;
  selectionGroups: string[][];
  issues: Array<{ nodeId: string; reason: WorkflowCaptureIssueReason }>;
  candidate: WorkflowCaptureCandidate | null;
  sourceDigest: string | null;
}

export function previewWorkflowCapture(
  spaceId: string,
  selectedNodeIds: string[],
): Promise<WorkflowCapturePreview> {
  return invoke("plugin:spaces|preview_workflow_capture", { spaceId, selectedNodeIds });
}

export interface SaveCapturedWorkflowRequest {
  requestId: string;
  spaceId: string;
  selectedNodeIds: string[];
  sourceDigest: string;
  name: string;
  outputNames: Record<string, string>;
}

export interface SavedCapturedWorkflow {
  schemaVersion: 3;
  workflowId: string;
  version: number;
  name: string;
  candidate: WorkflowCaptureCandidate;
  sourceSpaceId: string;
  sourceDigest: string;
  selectedNodeIds: string[];
  createdAt: number;
}

export function saveCapturedWorkflow(request: SaveCapturedWorkflowRequest): Promise<SavedCapturedWorkflow> {
  return invoke("plugin:spaces|save_captured_workflow", { request });
}

export function listCapturedWorkflows(): Promise<{
  items: SavedCapturedWorkflow[];
}> {
  return invoke("plugin:spaces|list_captured_workflows");
}

export function renameCapturedWorkflow(
  workflowId: string,
  expectedName: string,
  name: string,
): Promise<SavedCapturedWorkflow> {
  return invoke("plugin:spaces|rename_captured_workflow", { workflowId, expectedName, name });
}

export function reviseCapturedWorkflow(request: {
  workflowId: string;
  expectedVersion: number;
  outputSlots: WorkflowCaptureCandidate["outputSlots"];
  stepParameters: Record<string, Record<string, unknown>>;
  removedStepIds: string[];
}): Promise<SavedCapturedWorkflow> {
  return invoke("plugin:spaces|revise_captured_workflow", { request });
}

export function deleteCapturedWorkflow(workflowId: string, expectedName: string): Promise<void> {
  return invoke("plugin:spaces|delete_captured_workflow", { workflowId, expectedName });
}

export function startCapturedWorkflow(request: {
  requestId: string;
  workflowId: string;
  expectedVersion: number;
  spaceId: string;
  bindings: Record<string, string[]>;
}): Promise<RunSummary> {
  return invoke("plugin:spaces|start_captured_workflow", { request });
}

export function listCapturedRunOutputs(
  runId: string,
): Promise<{ outputSlotId: string; name: string; nodeId: string }[]> {
  return invoke("plugin:spaces|list_captured_run_outputs", { runId });
}

export interface CapturedWorkflowRun {
  run: RunSummary;
  workflowId: string;
}

export function listCapturedWorkflowRuns(spaceId: string): Promise<CapturedWorkflowRun[]> {
  return invoke("plugin:spaces|list_captured_workflow_runs", { spaceId });
}

export function retryCapturedWorkflowRun(runId: string): Promise<RunSummary> {
  return invoke("plugin:spaces|retry_run", { runId, stepIds: [] });
}

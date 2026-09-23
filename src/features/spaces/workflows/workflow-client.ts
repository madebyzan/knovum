import { invoke } from "@tauri-apps/api/core";

import type { WorkflowDefinition } from "../contracts/generated/WorkflowDefinition";
import type { WorkflowDraft } from "../contracts/generated/WorkflowDraft";
import type { WorkflowNode } from "../contracts/generated/WorkflowNode";
import type { WorkflowVersion } from "../contracts/generated/WorkflowVersion";

export type { WorkflowDefinition, WorkflowDraft, WorkflowNode, WorkflowVersion };
export type WorkflowNodeKind = WorkflowNode["kind"];
export interface WorkflowDiagnostic {
  code: string;
  message: string;
  nodeId: string | null;
  edgeId: string | null;
}
export interface WorkflowReadinessIssue {
  code: string;
  nodeId: string;
  message: string;
}
export interface StartWorkflowRequest {
  spaceId: string;
  workflowId: string;
  version: number;
  input: unknown;
  testMode?: boolean;
  breakpointNodeIds?: string[];
}
export interface StartWorkflowStepTestRequest {
  spaceId: string;
  workflowId: string;
  version: number;
  nodeId: string;
  input: unknown;
}

export function listWorkflowDrafts(spaceId: string): Promise<WorkflowDraft[]> {
  return invoke("plugin:spaces|list_workflow_drafts", { spaceId });
}

export function listWorkflowVersions(spaceId: string): Promise<WorkflowVersion[]> {
  return invoke("plugin:spaces|list_workflow_versions", { spaceId });
}

export function extractWorkflowDraftFromHistory(spaceId: string, title: string): Promise<WorkflowDraft> {
  return invoke("plugin:spaces|extract_workflow_draft_from_history", { spaceId, title });
}

export function saveWorkflowDraft(draft: WorkflowDraft): Promise<WorkflowDraft> {
  return invoke("plugin:spaces|save_workflow_draft", { draft });
}

export function publishWorkflow(workflowId: string, expectedRevision: number): Promise<WorkflowVersion> {
  return invoke("plugin:spaces|publish_workflow", { workflowId, expectedRevision });
}

export function validateWorkflow(definition: WorkflowDefinition): Promise<unknown> {
  return invoke("plugin:spaces|validate_workflow", { definition });
}

export function preflightWorkflow(definition: WorkflowDefinition): Promise<WorkflowReadinessIssue[]> {
  return invoke("plugin:spaces|preflight_workflow", { definition });
}

export function deleteWorkflowDraft(workflowId: string): Promise<void> {
  return invoke("plugin:spaces|delete_workflow_draft", { workflowId });
}

export function startWorkflow(request: StartWorkflowRequest): Promise<{ id: string }> {
  return invoke("plugin:spaces|start_workflow", {
    request: {
      ...request,
      parentRunId: null,
      grantId: null,
      testMode: request.testMode ?? false,
      breakpointNodeIds: request.breakpointNodeIds ?? [],
    },
  });
}

export function startWorkflowStepTest(request: StartWorkflowStepTestRequest): Promise<{ id: string }> {
  return invoke("plugin:spaces|start_workflow_step_test", {
    request,
  });
}

export function newWorkflowDraft(spaceId: string): WorkflowDraft {
  const id = crypto.randomUUID();
  return {
    id,
    spaceId,
    revision: 0,
    createdAt: 0,
    updatedAt: 0,
    definition: {
      schemaVersion: 1,
      title: "Untitled workflow",
      nodes: [
        { kind: "input", id: "input", outputType: "record" },
        { kind: "output", id: "output", inputType: "record" },
      ],
      edges: [
        {
          id: crypto.randomUUID(),
          sourceNodeId: "input",
          sourcePort: "output",
          targetNodeId: "output",
          targetPort: "input",
          branch: null,
        },
      ],
      layout: { input: { x: 40, y: 80 }, output: { x: 300, y: 80 } },
      failurePolicy: "stop_run",
      limits: {
        maxExpandedSteps: 1000,
        maxParallelism: 4,
        maxDurationSeconds: 3600,
        maxTotalTokens: 250_000,
        maxStepAttempts: 5_000,
        maxInputBytes: 67_108_864,
        maxOutputBytes: 268_435_456,
      },
    },
  };
}

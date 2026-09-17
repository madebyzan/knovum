export type CreationWorkflowStepStatus = "pending" | "running" | "succeeded" | "failed";

export interface CreationWorkflowStepDefinition<Action extends string = string> {
  id: string;
  action: Action;
  inputs: Readonly<Record<string, string>>;
}

export interface CreationWorkflowDefinition<Action extends string = string> {
  id: string;
  version: number;
  steps: readonly CreationWorkflowStepDefinition<Action>[];
}

export interface CreationWorkflowStepOutput<Output = unknown> {
  output: Output;
  outputNodeIds: readonly string[];
}

export interface CreationWorkflowStepSnapshot {
  id: string;
  action: string;
  status: CreationWorkflowStepStatus;
  outputNodeIds: readonly string[];
  error?: string;
}

export interface CreationWorkflowRunSnapshot {
  id: string;
  workflowId: string;
  workflowVersion: number;
  steps: readonly CreationWorkflowStepSnapshot[];
}

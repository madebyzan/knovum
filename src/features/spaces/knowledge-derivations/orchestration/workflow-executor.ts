import { invoke } from "@tauri-apps/api/core";
import { mapValues } from "es-toolkit";

import type {
  CreationWorkflowDefinition,
  CreationWorkflowRunSnapshot,
  CreationWorkflowStepDefinition,
  CreationWorkflowStepOutput,
  CreationWorkflowStepSnapshot,
} from "../workflows/workflow";

type StepOutputs<Output> = ReadonlyMap<string, CreationWorkflowStepOutput<Output>>;
export interface CreationStepContext<Action extends string, Output> {
  step: CreationWorkflowStepDefinition<Action>;
  inputs: Readonly<Record<string, CreationWorkflowStepOutput<Output>>>;
}
export type CreationStepAdapter<Action extends string, Output> = (
  context: CreationStepContext<Action, Output>,
) => Promise<CreationWorkflowStepOutput<Output>>;
export type CreationStepAdapters<Action extends string, Output> = Record<
  Action,
  CreationStepAdapter<Action, Output>
>;

export interface WorkflowExecutionOptions<Action extends string, Output> {
  runId: string;
  spaceId: string;
  workflow: CreationWorkflowDefinition<Action>;
  adapters: CreationStepAdapters<Action, Output>;
  onChange?: (snapshot: CreationWorkflowRunSnapshot) => Promise<void> | void;
}

export async function executeCreationWorkflow<Action extends string, Output>(
  options: WorkflowExecutionOptions<Action, Output>,
): Promise<CreationWorkflowRunSnapshot> {
  const completed = new Map<string, CreationWorkflowStepOutput<Output>>();
  const snapshots = new Map<string, CreationWorkflowStepSnapshot>(
    options.workflow.steps.map((step) => [step.id, stepSnapshot(step)]),
  );
  await publish(options, snapshots);
  return executePendingSteps(options, completed, snapshots);
}

async function executePendingSteps<Action extends string, Output>(
  options: WorkflowExecutionOptions<Action, Output>,
  completed: Map<string, CreationWorkflowStepOutput<Output>>,
  snapshots: Map<string, CreationWorkflowStepSnapshot>,
): Promise<CreationWorkflowRunSnapshot> {
  if (completed.size === options.workflow.steps.length) {
    return snapshot(options.runId, options.workflow, snapshots);
  }
  const stepId = await invoke<string | null>("plugin:spaces|plan_creation_workflow_step", {
    request: {
      id: options.runId,
      spaceId: options.spaceId,
      workflowId: options.workflow.id,
      workflowVersion: options.workflow.version,
      steps: [...snapshots.values()],
    },
  });
  const step = options.workflow.steps.find((candidate) => candidate.id === stepId);
  if (!step) throw new Error("Creation Workflow has no executable step");

  snapshots.set(step.id, { ...stepSnapshot(step), status: "running" });
  await publish(options, snapshots);
  try {
    const output = await options.adapters[step.action]({
      step,
      inputs: resolveInputs(step, completed),
    });
    completed.set(step.id, output);
    snapshots.set(step.id, {
      ...stepSnapshot(step),
      status: "succeeded",
      outputNodeIds: output.outputNodeIds,
    });
  } catch (error) {
    snapshots.set(step.id, {
      ...stepSnapshot(step),
      status: "failed",
      error: String(error),
    });
    await publish(options, snapshots);
    throw error;
  }
  await publish(options, snapshots);
  return executePendingSteps(options, completed, snapshots);
}

function stepSnapshot(step: CreationWorkflowStepDefinition): CreationWorkflowStepSnapshot {
  return { id: step.id, action: step.action, status: "pending", outputNodeIds: [] };
}

function resolveInputs<Action extends string, Output>(
  step: CreationWorkflowStepDefinition<Action>,
  completed: StepOutputs<Output>,
) {
  return mapValues(step.inputs, (stepId) => completed.get(stepId)!);
}

async function publish<Action extends string, Output>(
  options: WorkflowExecutionOptions<Action, Output>,
  steps: ReadonlyMap<string, CreationWorkflowStepSnapshot>,
) {
  await options.onChange?.(snapshot(options.runId, options.workflow, steps));
}

function snapshot(
  id: string,
  workflow: CreationWorkflowDefinition,
  steps: ReadonlyMap<string, CreationWorkflowStepSnapshot>,
): CreationWorkflowRunSnapshot {
  return { id, workflowId: workflow.id, workflowVersion: workflow.version, steps: [...steps.values()] };
}

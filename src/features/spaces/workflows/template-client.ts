import { invoke } from "@tauri-apps/api/core";

import type { ExecutionTarget } from "../../../foundation/ai/ai-models";
import type { BuiltInWorkflowTemplateId } from "../contracts/generated/BuiltInWorkflowTemplateId";
import type { InstantiateWorkflowTemplateRequest } from "../contracts/generated/InstantiateWorkflowTemplateRequest";
import type { WorkflowVersion } from "../contracts/generated/WorkflowVersion";
import { startWorkflow } from "./workflow-client";

interface TemplateOptions {
  spaceId: string;
  templateId: BuiltInWorkflowTemplateId;
  inputNodeIds: readonly string[];
  parameters: InstantiateWorkflowTemplateRequest["parameters"];
  targets?: Record<string, ExecutionTarget | undefined>;
}

export async function runWorkflowTemplate(options: TemplateOptions) {
  const request: InstantiateWorkflowTemplateRequest = {
    spaceId: options.spaceId,
    templateId: options.templateId,
    parameters: options.parameters,
  };
  const version = await invoke<WorkflowVersion>("plugin:spaces|instantiate_workflow_template", {
    request,
  });
  const targets = Object.fromEntries(
    Object.entries(options.targets ?? {})
      .filter((entry): entry is [string, ExecutionTarget] => Boolean(entry[1]))
      .map(([actionId, target]) => [actionId, { ...target, temperature: 0.2 }]),
  );
  return startWorkflow({
    spaceId: options.spaceId,
    workflowId: version.workflowId,
    version: version.version,
    input: { inputNodeIds: options.inputNodeIds, targets },
  });
}

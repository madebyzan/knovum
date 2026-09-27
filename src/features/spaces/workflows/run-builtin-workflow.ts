import type { ExecutionTarget } from "../../../foundation/ai/ai-models";
import { i18n } from "../../../foundation/i18n/i18n";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { BuiltInWorkflowTemplateId } from "../contracts/generated/BuiltInWorkflowTemplateId";
import type { RunObservation } from "../contracts/generated/RunObservation";
import { observeWorkflowRun } from "./workflow-run-observation";

interface BuiltinWorkflowOptions {
  spaceId: string;
  templateId: BuiltInWorkflowTemplateId;
  inputNodeIds: readonly string[];
  parameters: Record<string, unknown>;
  targets?: Record<string, ExecutionTarget | undefined>;
}

export async function startBuiltinWorkflow(options: BuiltinWorkflowOptions) {
  const targets = Object.fromEntries(
    Object.entries(options.targets ?? {})
      .filter((entry): entry is [string, ExecutionTarget] => Boolean(entry[1]))
      .map(([actionId, target]) => [actionId, { ...target, temperature: 0.2 }]),
  );
  const run = await invokeCommand<RunObservation>("plugin:spaces|start_builtin_workflow", {
    request: {
      requestId: crypto.randomUUID(),
      spaceId: options.spaceId,
      templateId: options.templateId,
      parameters: options.parameters,
      inputNodeIds: options.inputNodeIds,
      targets,
    },
  });
  observeWorkflowRun(run, i18n.global.t("spaces.creation.workflow.title"));
  return run;
}

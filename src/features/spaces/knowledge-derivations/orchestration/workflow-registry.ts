import { invoke } from "@tauri-apps/api/core";
import { z } from "zod";

import type { CreationWorkflowDefinition } from "../workflows/workflow";

const workflowDefinitionSchema = z
  .object({
    id: z.string().min(1),
    version: z.number().int().positive(),
    steps: z
      .array(
        z
          .object({
            id: z.string().min(1),
            action: z.string().min(1),
            dependsOn: z.array(z.string().min(1)),
          })
          .strict(),
      )
      .min(1),
    parameters: z.array(z.object({ key: z.string().min(1), kind: z.enum(["text", "number"]) }).strict()),
  })
  .strict();

let definitions: Promise<z.infer<typeof workflowDefinitionSchema>[]> | undefined;

export async function loadWorkflow<Action extends string>(
  id: string,
): Promise<CreationWorkflowDefinition<Action>> {
  definitions ??= invoke<unknown>("plugin:spaces|list_creation_workflows").then((value) =>
    z.array(workflowDefinitionSchema).parse(value),
  );
  const definition = (await definitions).find((candidate) => candidate.id === id);
  if (!definition) throw new Error(`Creation Workflow is not registered: ${id}`);
  return {
    id: definition.id,
    version: definition.version,
    steps: definition.steps.map((step) => ({
      id: step.id,
      action: step.action as Action,
      inputs: Object.fromEntries(step.dependsOn.map((dependency) => [dependency, dependency])),
    })),
  };
}

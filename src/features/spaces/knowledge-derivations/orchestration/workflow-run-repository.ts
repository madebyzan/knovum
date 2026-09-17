import { invoke } from "@tauri-apps/api/core";

import type { CreationWorkflowRunSnapshot } from "../workflows/workflow";

export async function saveCreationWorkflowRun(spaceId: string, snapshot: CreationWorkflowRunSnapshot) {
  await invoke("plugin:spaces|save_creation_workflow_run", {
    request: {
      id: snapshot.id,
      spaceId,
      workflowId: snapshot.workflowId,
      workflowVersion: snapshot.workflowVersion,
      steps: snapshot.steps.map((step) => ({
        id: step.id,
        action: step.action,
        status: step.status,
        outputNodeIds: step.outputNodeIds,
        ...(step.error === undefined ? {} : { error: step.error }),
      })),
    },
  });
}

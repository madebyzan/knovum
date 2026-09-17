import { requiredMap } from "./id-maps";
import type { SpacePackageRestoreContext } from "./restore-context";
import type { SpacePackageRestoreRequest } from "./restore-transport";

export function restoreWorkflowRuns(
  context: SpacePackageRestoreContext,
): SpacePackageRestoreRequest["creationWorkflowRuns"] {
  const { maps } = context;
  return context.manifest.creationWorkflowRuns.map((run) => ({
    ...run,
    id: requiredMap(maps.workflowRuns, run.id, "Creation Workflow run"),
    steps: run.steps.map((step) => ({
      ...step,
      outputNodeIds: step.outputNodeIds.map((id) => requiredMap(maps.nodes, id, "Creation Workflow output")),
    })),
  }));
}

import type { SpacePackageManifest } from "./manifest-schema";

export function validatePackageWorkflowRuns(manifest: SpacePackageManifest) {
  const nodeIds = new Set(manifest.nodes.map(({ id }) => id));
  for (const run of manifest.creationWorkflowRuns) {
    for (const step of run.steps) {
      if (step.outputNodeIds.some((id) => !nodeIds.has(id))) {
        throw new Error("Creation Workflow step references an unavailable output node");
      }
    }
  }
}

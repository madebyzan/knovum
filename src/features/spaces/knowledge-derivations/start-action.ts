import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { TaskTargetRef } from "../../../shared/contracts/generated/TaskTargetRef";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { StartSpaceActionRequest } from "../contracts/generated/StartSpaceActionRequest";
import { observeResourceRun } from "../runs/resource-run-observation";
import { useSpaceStore } from "../session/space-store";
import type { GroundedDocumentSource } from "./catalog/grounded-document";

interface StartActionOptions {
  actionId: string;
  sources: readonly GroundedDocumentSource[];
  parameters: Record<string, unknown>;
  title: string;
  target?: TaskTargetRef | undefined;
}

export async function startDerivationAction({
  actionId,
  sources,
  parameters,
  title,
  target,
}: StartActionOptions) {
  const store = useSpaceStore();

  const spaceId = store.activeSpaceId;
  if (!spaceId) return;
  const request: StartSpaceActionRequest = {
    requestId: crypto.randomUUID(),
    spaceId,
    actionId,
    parameters,
    sources: sources.map(({ nodeId, sourceRevision }) => ({ nodeId, knowledgeRevision: sourceRevision })),
    target: target ? { ...target } : null,
  };
  try {
    const run = await invokeCommand<RunObservation>("plugin:spaces|start_space_action", { request });
    observeResourceRun(run, title);
    return run;
  } catch (error) {
    toast.error(getErrorMessage(error));
    return undefined;
  }
}

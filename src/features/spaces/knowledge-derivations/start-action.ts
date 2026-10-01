import { toast } from "vue-sonner";

import { errorMessage } from "../../../foundation/errors/error-format";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { TaskTargetRef } from "../../../shared/contracts/generated/TaskTargetRef";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { StartSpaceActionRequest } from "../contracts/generated/StartSpaceActionRequest";
import { observeResourceRun } from "../runs/resource-run-observation";
import type { useSpaceStore } from "../session/space-store";
import type { GroundedDocumentSource } from "./catalog/grounded-document";

export type DerivationActionStore = Pick<ReturnType<typeof useSpaceStore>, "activeSpaceId">;

interface StartActionOptions {
  store: DerivationActionStore;
  actionId: string;
  sources: readonly GroundedDocumentSource[];
  parameters: Record<string, unknown>;
  title: string;
  target?: TaskTargetRef | undefined;
}

export async function startDerivationAction({
  store,
  actionId,
  sources,
  parameters,
  title,
  target,
}: StartActionOptions) {
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
    toast.error(errorMessage(error));
    return undefined;
  }
}

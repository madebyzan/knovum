import { readonly, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { CreateSpeechProjectRequest } from "../contracts/generated/CreateSpeechProjectRequest";
import { createSpeechProject, getSpeechProject } from "./ipc";

interface ProjectCreationOwner {
  activeSpaceId: () => string | undefined;
  sourceNodeId: () => string;
  contextEpoch: () => number;
  isCurrent: (spaceId: string | undefined, epoch: number) => boolean;
}

export function useSpeechProjectCreation(owner: ProjectCreationOwner) {
  const creating = shallowRef(false);
  const error = shallowRef<string>();
  let operationEpoch = 0;

  function isCurrentOperation(operation: number, spaceId: string | undefined, epoch: number) {
    return operation === operationEpoch && owner.isCurrent(spaceId, epoch);
  }

  async function add(input: Omit<CreateSpeechProjectRequest, "spaceId" | "sourceNodeId">) {
    if (creating.value) return;
    const operation = ++operationEpoch;
    const spaceId = owner.activeSpaceId();
    const epoch = owner.contextEpoch();
    creating.value = true;
    error.value = undefined;
    try {
      if (!spaceId) throw new Error("A persisted Space is required");
      const sourceNodeId = owner.sourceNodeId();
      const project = await createSpeechProject({ ...input, spaceId, sourceNodeId });
      const snapshot = await getSpeechProject(project.id);
      if (
        !isCurrentOperation(operation, spaceId, epoch) ||
        snapshot.project.spaceId !== spaceId ||
        snapshot.project.sourceNodeId !== sourceNodeId
      )
        return;
      return { project, snapshot };
    } catch (cause) {
      if (isCurrentOperation(operation, spaceId, epoch)) error.value = getErrorMessage(cause);
    } finally {
      if (operation === operationEpoch) creating.value = false;
    }
  }

  function reset() {
    operationEpoch += 1;
    creating.value = false;
    error.value = undefined;
  }

  return { creating: readonly(creating), error: readonly(error), add, reset };
}

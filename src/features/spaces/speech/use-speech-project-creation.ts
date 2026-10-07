import { readonly, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { CreateSpeechProjectRequest } from "../contracts/generated/CreateSpeechProjectRequest";
import type { SpeechProjectOverview } from "../contracts/generated/SpeechProjectOverview";
import type { SpeechProjectSnapshot } from "../contracts/generated/SpeechProjectSnapshot";
import { createSpeechProject, getSpeechProject } from "./ipc";

interface ProjectCreationOwner {
  activeSpaceId: () => string | undefined;
  sourceNodeId: () => string;
  contextEpoch: () => number;
  isCurrent: (spaceId: string | undefined, epoch: number) => boolean;
  created: (summary: SpeechProjectOverview, snapshot: SpeechProjectSnapshot) => void;
  setError: (message?: string) => void;
}

export function useSpeechProjectCreation(owner: ProjectCreationOwner) {
  const creating = shallowRef(false);

  async function add(input: Omit<CreateSpeechProjectRequest, "spaceId" | "sourceNodeId">) {
    if (creating.value) return;
    const spaceId = owner.activeSpaceId();
    const epoch = owner.contextEpoch();
    creating.value = true;
    try {
      if (!spaceId) throw new Error("A persisted Space is required");
      const sourceNodeId = owner.sourceNodeId();
      const summary = await createSpeechProject({ ...input, spaceId, sourceNodeId });
      const snapshot = await getSpeechProject(summary.id);
      if (
        !owner.isCurrent(spaceId, epoch) ||
        snapshot.project.spaceId !== spaceId ||
        snapshot.project.sourceNodeId !== sourceNodeId
      )
        return;
      owner.created(summary, snapshot);
      owner.setError();
    } catch (cause) {
      if (owner.isCurrent(spaceId, epoch)) owner.setError(getErrorMessage(cause));
    } finally {
      creating.value = false;
    }
  }

  return { creating: readonly(creating), add };
}

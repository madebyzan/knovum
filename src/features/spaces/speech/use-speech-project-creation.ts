import { readonly, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { CreateSpeechProjectInput, SpeechProjectDetail, SpeechProjectSummary } from "./contracts";
import { createSpeechProject, getSpeechProject } from "./ipc";

interface ProjectCreationOwner {
  activeSpaceId: () => string | undefined;
  contextEpoch: () => number;
  isCurrent: (spaceId: string | undefined, epoch: number) => boolean;
  created: (summary: SpeechProjectSummary, detail: SpeechProjectDetail) => void;
  setError: (message?: string) => void;
}

export function useSpeechProjectCreation(owner: ProjectCreationOwner) {
  const creating = shallowRef(false);

  async function add(input: Omit<CreateSpeechProjectInput, "spaceId">) {
    if (creating.value) return;
    const spaceId = owner.activeSpaceId();
    const epoch = owner.contextEpoch();
    creating.value = true;
    try {
      if (!spaceId) throw new Error("A persisted Space is required");
      const summary = await createSpeechProject({ ...input, spaceId });
      const detail = await getSpeechProject(summary.id);
      if (!owner.isCurrent(spaceId, epoch) || detail.project.spaceId !== spaceId) return;
      owner.created(summary, detail);
      owner.setError();
    } catch (cause) {
      if (owner.isCurrent(spaceId, epoch)) owner.setError(getErrorMessage(cause));
    } finally {
      creating.value = false;
    }
  }

  return { creating: readonly(creating), add };
}

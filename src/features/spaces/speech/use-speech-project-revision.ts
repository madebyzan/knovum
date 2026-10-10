import { computed, onScopeDispose, readonly, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { SaveSpeechProjectRevisionRequest } from "../contracts/generated/SaveSpeechProjectRevisionRequest";
import type { SpeechProjectSnapshot } from "../contracts/generated/SpeechProjectSnapshot";
import { useSpaceStore } from "../session/space-store";
import { saveSpeechProjectRevision } from "./ipc";

export type SpeechRevisionInput = Omit<SaveSpeechProjectRevisionRequest, "projectId" | "expectedRevision">;

export function useSpeechProjectRevision() {
  const store = useSpaceStore();
  const selectedProject = shallowRef<SpeechProjectSnapshot>();
  const pending = shallowRef<Promise<boolean>>();
  const error = shallowRef<string>();
  let selectionEpoch = 0;
  function select(snapshot?: SpeechProjectSnapshot) {
    selectionEpoch += 1;
    selectedProject.value = snapshot;
    error.value = undefined;
  }
  function isCurrent(spaceId: string, epoch: number, projectId: string) {
    return (
      store.activeSpaceId === spaceId &&
      selectionEpoch === epoch &&
      selectedProject.value?.project.id === projectId
    );
  }
  function save(input: SpeechRevisionInput) {
    pending.value ??= persist(input).finally(() => {
      pending.value = undefined;
    });
    return pending.value;
  }
  async function persist(input: SpeechRevisionInput) {
    const current = selectedProject.value;
    const spaceId = store.activeSpaceId;
    const epoch = selectionEpoch;
    if (!current || !spaceId || current.project.spaceId !== spaceId) return false;
    try {
      const snapshot = await saveSpeechProjectRevision({
        ...input,
        projectId: current.project.id,
        expectedRevision: current.project.activeRevision,
      });
      if (!isCurrent(spaceId, epoch, current.project.id) || snapshot.project.spaceId !== spaceId)
        return false;
      selectedProject.value = snapshot;
      error.value = undefined;
      return true;
    } catch (cause) {
      if (isCurrent(spaceId, epoch, current.project.id)) error.value = getErrorMessage(cause);
      return false;
    }
  }
  async function settle() {
    await pending.value;
  }
  onScopeDispose(() => {
    selectionEpoch += 1;
  });
  return {
    selectedProject: readonly(selectedProject),
    saving: computed(() => Boolean(pending.value)),
    error: readonly(error),
    select,
    save,
    settle,
  };
}

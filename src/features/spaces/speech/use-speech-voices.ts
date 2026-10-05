import { readonly, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { CreatePresetVoiceInput, VoiceLibraryRevision } from "./contracts";
import { createPresetVoice, listVoiceRevisions } from "./ipc";

interface VoiceLibraryOwner {
  activeSpaceId: () => string | undefined;
  contextEpoch: () => number;
  isCurrent: (spaceId: string | undefined, epoch: number) => boolean;
  setError: (message?: string) => void;
}

export function useSpeechVoices(owner: VoiceLibraryOwner) {
  const voices = shallowRef<VoiceLibraryRevision[]>([]);
  const creating = shallowRef(false);

  function replace(next: VoiceLibraryRevision[]) {
    voices.value = next;
  }

  function accept(voice: VoiceLibraryRevision) {
    if (!owner.activeSpaceId() || voices.value.some((current) => current.revisionId === voice.revisionId))
      return;
    voices.value = [voice, ...voices.value];
  }

  async function addPreset(input: Omit<CreatePresetVoiceInput, "spaceId">) {
    if (creating.value) return;
    const spaceId = owner.activeSpaceId();
    const epoch = owner.contextEpoch();
    creating.value = true;
    try {
      if (!spaceId) throw new Error("A persisted Space is required");
      const voice = await createPresetVoice({ ...input, spaceId });
      if (!owner.isCurrent(spaceId, epoch)) return;
      accept(voice);
      owner.setError();
    } catch (cause) {
      if (owner.isCurrent(spaceId, epoch)) owner.setError(getErrorMessage(cause));
    } finally {
      creating.value = false;
    }
  }

  async function refresh() {
    const spaceId = owner.activeSpaceId();
    const epoch = owner.contextEpoch();
    if (!spaceId) return;
    try {
      const next = await listVoiceRevisions(spaceId);
      if (owner.isCurrent(spaceId, epoch)) replace(next);
    } catch (cause) {
      if (owner.isCurrent(spaceId, epoch)) owner.setError(getErrorMessage(cause));
    }
  }

  return {
    voices: readonly(voices),
    creating: readonly(creating),
    load: listVoiceRevisions,
    replace,
    accept,
    addPreset,
    refresh,
  };
}

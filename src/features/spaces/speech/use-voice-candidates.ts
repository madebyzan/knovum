import { computed, onScopeDispose, readonly, shallowRef, watch } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { RunObservation } from "../contracts/generated/RunObservation";
import { cancelResourceRun } from "../runs/resource-run-observation";
import { useSpaceStore } from "../session/space-store";
import type { DraftVoiceCandidateRequest, VoiceLibraryRevision } from "./contracts";
import { draftVoiceCandidate, getActiveVoiceCandidateRun, listVoiceCandidates } from "./voice-candidate-ipc";
import { observeVoiceCandidateRun } from "./voice-candidate-run";
import {
  auditionVoiceCandidate,
  type PlayableVoiceCandidate,
  presentVoiceCandidate,
  rejectVoiceCandidate,
  selectVoiceCandidate,
} from "./voice-candidate-selection";

export function useVoiceCandidates(created: (voice: VoiceLibraryRevision) => void) {
  const store = useSpaceStore();
  const candidates = shallowRef<PlayableVoiceCandidate[]>([]);
  const activeRunId = shallowRef<string>();
  const submitting = shallowRef(false);
  const error = shallowRef<string>();
  const busy = computed(() => Boolean(activeRunId.value) || submitting.value);
  let epoch = 0;
  let stopObservation: (() => unknown) | undefined;

  function reset() {
    epoch += 1;
    stopObservation?.();
    stopObservation = undefined;
    candidates.value = [];
    activeRunId.value = undefined;
    submitting.value = false;
    error.value = undefined;
  }
  function current(spaceId: string, version: number) {
    return store.activeSpaceId === spaceId && epoch === version;
  }
  async function reload(spaceId: string, version: number) {
    const loaded = await listVoiceCandidates(spaceId);
    if (current(spaceId, version)) candidates.value = loaded.map(presentVoiceCandidate);
  }
  function observe(run: RunObservation, spaceId: string, version: number) {
    stopObservation?.();
    activeRunId.value = run.id;
    const settle = () => {
      if (!current(spaceId, version)) return;
      stopObservation?.();
      stopObservation = undefined;
      activeRunId.value = undefined;
    };
    stopObservation = observeVoiceCandidateRun(
      run,
      spaceId,
      (loaded) => {
        if (!current(spaceId, version)) return;
        settle();
        candidates.value = [presentVoiceCandidate(loaded), ...candidates.value];
      },
      (message) => {
        settle();
        if (current(spaceId, version)) error.value = message;
      },
    );
  }
  async function operation(action: (spaceId: string, version: number) => Promise<void>) {
    const spaceId = store.activeSpaceId;
    const version = epoch;
    if (!spaceId || busy.value) return;
    submitting.value = true;
    error.value = undefined;
    try {
      await action(spaceId, version);
    } catch (cause) {
      if (current(spaceId, version)) error.value = getErrorMessage(cause);
    } finally {
      if (current(spaceId, version)) submitting.value = false;
    }
  }
  function selected(id: string) {
    const candidate = candidates.value.find((item) => item.candidateId === id);
    if (!candidate) throw new Error("Voice candidate is no longer available");
    return candidate;
  }
  const draft = (input: Omit<DraftVoiceCandidateRequest, "requestId" | "spaceId">) =>
    operation(async (spaceId, version) => {
      const run = await draftVoiceCandidate({ ...input, requestId: crypto.randomUUID(), spaceId });
      if (current(spaceId, version)) observe(run, spaceId, version);
    });
  async function audition(id: string) {
    const spaceId = store.activeSpaceId;
    const version = epoch;
    if (!spaceId) return;
    try {
      const updated = await auditionVoiceCandidate(selected(id));
      if (current(spaceId, version))
        candidates.value = candidates.value.map((item) => (item.candidateId === id ? updated : item));
    } catch (cause) {
      if (current(spaceId, version)) error.value = getErrorMessage(cause);
    }
  }
  const save = (id: string, name: string, authorized: boolean) =>
    operation(async (spaceId, version) => {
      const voice = await selectVoiceCandidate(selected(id), name, authorized);
      if (!current(spaceId, version)) return;
      candidates.value = candidates.value.filter((item) => item.candidateId !== id);
      created(voice);
    });
  const discard = (id: string) =>
    operation(async (spaceId, version) => {
      await rejectVoiceCandidate(selected(id));
      if (current(spaceId, version))
        candidates.value = candidates.value.filter((item) => item.candidateId !== id);
    });
  async function cancel() {
    if (!activeRunId.value) return;
    try {
      await cancelResourceRun(activeRunId.value);
    } catch (cause) {
      error.value = getErrorMessage(cause);
    }
  }
  watch(
    () => store.activeSpaceId,
    (spaceId) => {
      reset();
      if (!spaceId) return;
      void operation(async (owner, version) => {
        const [run] = await Promise.all([getActiveVoiceCandidateRun(owner), reload(owner, version)]);
        if (run && current(owner, version)) observe(run, owner, version);
      });
    },
    { immediate: true },
  );
  onScopeDispose(reset);
  return {
    candidates: readonly(candidates),
    activeRunId: readonly(activeRunId),
    busy,
    error: readonly(error),
    draft,
    save,
    cancel,
    discard,
    audition,
  };
}

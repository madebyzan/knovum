import { computed, onScopeDispose, readonly, shallowRef, watch } from "vue";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { RunObservation } from "../contracts/generated/RunObservation";
import { cancelResourceRun } from "../runs/resource-run-observation";
import { useSpaceStore } from "../session/space-store";
import type { DraftVoiceCandidateRequest, VoiceProfileSummary } from "./contracts";
import {
  draftVoiceCandidate,
  getActiveVoiceCandidateRun,
  getLatestVoiceCandidate,
} from "./voice-candidate-ipc";
import { observeVoiceCandidateRun } from "./voice-candidate-run";
import {
  auditionVoiceCandidate,
  type PlayableVoiceCandidate,
  presentVoiceCandidate,
  rejectVoiceCandidate,
  selectVoiceCandidate,
} from "./voice-candidate-selection";

export function useVoiceCandidates(created: (voice: VoiceProfileSummary) => void) {
  const store = useSpaceStore();
  const candidate = shallowRef<PlayableVoiceCandidate>();
  const activeRunId = shallowRef<string>();
  const saving = shallowRef(false);
  const restoring = shallowRef(false);
  const error = shallowRef<string>();
  const busy = computed(() => Boolean(activeRunId.value) || saving.value || restoring.value);
  let stopObservation: (() => unknown) | undefined;
  let epoch = 0;

  function resetLocal() {
    epoch += 1;
    stopObservation?.();
    stopObservation = undefined;
    candidate.value = undefined;
    activeRunId.value = undefined;
    saving.value = false;
    restoring.value = false;
    error.value = undefined;
  }

  async function restore(spaceId: string) {
    const current = ++epoch;
    restoring.value = true;
    try {
      const run = await getActiveVoiceCandidateRun(spaceId);
      if (current !== epoch || store.activeSpaceId !== spaceId) return;
      if (run) {
        activeRunId.value = run.id;
        observe(run, current, spaceId);
      } else {
        const loaded = await getLatestVoiceCandidate(spaceId);
        if (current === epoch && loaded) candidate.value = presentVoiceCandidate(loaded);
      }
    } catch (cause) {
      if (current === epoch) error.value = getErrorMessage(cause);
    } finally {
      if (current === epoch) restoring.value = false;
    }
  }

  async function draft(input: Omit<DraftVoiceCandidateRequest, "requestId" | "spaceId">) {
    const spaceId = store.activeSpaceId;
    if (!spaceId || busy.value || candidate.value) return;
    const current = ++epoch;
    error.value = undefined;
    try {
      const run = await draftVoiceCandidate({
        ...input,
        requestId: crypto.randomUUID(),
        spaceId,
      });
      if (current !== epoch || store.activeSpaceId !== spaceId) return;
      activeRunId.value = run.id;
      observe(run, current, spaceId);
    } catch (cause) {
      if (current === epoch && decodeAppError(cause)?.code !== "cancelled") {
        error.value = getErrorMessage(cause);
      }
    }
  }

  function observe(run: RunObservation, current: number, spaceId: string) {
    stopObservation?.();
    const settle = () => {
      stopObservation?.();
      stopObservation = undefined;
      activeRunId.value = undefined;
    };
    stopObservation = observeVoiceCandidateRun(
      run,
      spaceId,
      (loaded) => {
        if (current !== epoch || store.activeSpaceId !== spaceId) return;
        settle();
        candidate.value = presentVoiceCandidate(loaded);
      },
      (message) => {
        if (current !== epoch) return;
        settle();
        error.value = message;
      },
    );
  }

  function ownsCandidate(candidateId: string, spaceId: string, current: number) {
    return (
      current === epoch && store.activeSpaceId === spaceId && candidate.value?.candidateId === candidateId
    );
  }

  async function audition() {
    const current = candidate.value;
    if (!current || current.auditionedAt !== null) return;
    const currentEpoch = epoch;
    const { candidateId, spaceId } = current;
    try {
      const auditioned = await auditionVoiceCandidate(current);
      if (ownsCandidate(candidateId, spaceId, currentEpoch)) candidate.value = auditioned;
    } catch (cause) {
      if (ownsCandidate(candidateId, spaceId, currentEpoch)) error.value = getErrorMessage(cause);
    }
  }

  async function save(name: string, authorized: boolean) {
    const current = candidate.value;
    const currentEpoch = epoch;
    if (!current || saving.value || current.auditionedAt === null) return;
    saving.value = true;
    error.value = undefined;
    try {
      const voice = await selectVoiceCandidate(current, name, authorized);
      if (currentEpoch === epoch && store.activeSpaceId === current.spaceId) {
        candidate.value = undefined;
        created(voice);
      }
    } catch (cause) {
      if (currentEpoch === epoch) error.value = getErrorMessage(cause);
    } finally {
      if (currentEpoch === epoch) saving.value = false;
    }
  }

  async function cancel() {
    if (!activeRunId.value) return;
    try {
      await cancelResourceRun(activeRunId.value);
    } catch (cause) {
      error.value = getErrorMessage(cause);
    }
  }

  async function discard() {
    const current = candidate.value;
    if (!current) return;
    const currentEpoch = epoch;
    const { candidateId, spaceId } = current;
    try {
      await rejectVoiceCandidate(current);
      if (ownsCandidate(candidateId, spaceId, currentEpoch)) candidate.value = undefined;
    } catch (cause) {
      if (ownsCandidate(candidateId, spaceId, currentEpoch)) error.value = getErrorMessage(cause);
    }
  }

  watch(
    () => store.activeSpaceId,
    (spaceId) => {
      resetLocal();
      if (spaceId) void restore(spaceId);
    },
    { immediate: true },
  );
  onScopeDispose(resetLocal);
  return {
    candidate: readonly(candidate),
    activeRunId: readonly(activeRunId),
    saving: readonly(saving),
    busy,
    error: readonly(error),
    draft,
    save,
    cancel,
    discard,
    audition,
  };
}

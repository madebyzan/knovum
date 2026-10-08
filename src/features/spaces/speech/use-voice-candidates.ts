import { computed, onScopeDispose, readonly, shallowRef, watch } from "vue";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { DraftVoiceCandidateRequest } from "../contracts/generated/DraftVoiceCandidateRequest";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { VoiceLibraryRevision } from "../contracts/generated/VoiceLibraryRevision";
import { cancelResourceRun } from "../runs/resource-run-observation";
import { useVoiceCandidateSubmission } from "./use-voice-candidate-submission";
import { getActiveVoiceCandidateRun, listVoiceCandidates } from "./voice-candidate-ipc";
import { observeVoiceCandidateRun } from "./voice-candidate-run";
import {
  auditionVoiceCandidate,
  type PlayableVoiceCandidate,
  presentVoiceCandidate,
  rejectVoiceCandidate,
  selectVoiceCandidate,
} from "./voice-candidate-selection";

export function useVoiceCandidates(
  ownerSpaceId: () => string | undefined,
  created: (voice: VoiceLibraryRevision) => void,
) {
  const candidates = shallowRef<PlayableVoiceCandidate[]>([]);
  const activeRunId = shallowRef<string>();
  const submitting = shallowRef(false);
  const error = shallowRef<string>();
  const busy = computed(() => Boolean(activeRunId.value) || submitting.value);
  const submission = useVoiceCandidateSubmission();
  let epoch = 0;
  let stopObservation: (() => unknown) | undefined;

  function reset() {
    epoch += 1;
    stopObservation?.();
    stopObservation = undefined;
    candidates.value = [];
    activeRunId.value = undefined;
    submission.reset();
    submitting.value = false;
    error.value = undefined;
  }
  function current(spaceId: string, version: number) {
    return ownerSpaceId() === spaceId && epoch === version;
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
    const spaceId = ownerSpaceId();
    const version = epoch;
    if (!spaceId || busy.value) return;
    submitting.value = true;
    error.value = undefined;
    try {
      await action(spaceId, version);
    } catch (cause) {
      if (current(spaceId, version) && decodeAppError(cause)?.code !== "cancelled")
        error.value = getErrorMessage(cause);
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
      await submission.start({ ...input, spaceId }, (run) => {
        if (current(spaceId, version)) observe(run, spaceId, version);
      });
    });
  async function audition(id: string) {
    const spaceId = ownerSpaceId();
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
    const spaceId = ownerSpaceId();
    const version = epoch;
    const runId = activeRunId.value ?? submission.jobId.value;
    if (!spaceId || !runId) return;
    try {
      if (activeRunId.value) await cancelResourceRun(runId);
      else await submission.cancel();
    } catch (cause) {
      if (current(spaceId, version) && (activeRunId.value === runId || submission.jobId.value === runId))
        error.value = getErrorMessage(cause);
    }
  }
  watch(
    () => ownerSpaceId(),
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
    pendingRequestId: submission.jobId,
    busy,
    error: readonly(error),
    draft,
    save,
    cancel,
    discard,
    audition,
  };
}

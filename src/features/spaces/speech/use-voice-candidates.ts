import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { computed, onScopeDispose, shallowRef, watch } from "vue";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { DraftVoiceCandidateRequest } from "../contracts/generated/DraftVoiceCandidateRequest";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { VoiceCandidate } from "../contracts/generated/VoiceCandidate";
import type { VoiceLibraryRevision } from "../contracts/generated/VoiceLibraryRevision";
import { cancelResourceRun } from "../runs/resource-run-observation";
import { useVoiceCandidateSubmission } from "./use-voice-candidate-submission";
import { getActiveVoiceCandidateRun, listVoiceCandidates } from "./voice-candidate-ipc";
import { observeVoiceCandidateRun } from "./voice-candidate-run";
import {
  auditionVoiceCandidate,
  presentVoiceCandidate,
  rejectVoiceCandidate,
  selectVoiceCandidate,
} from "./voice-candidate-selection";
import { VOICE_QUERY_KEYS } from "./voice-query-keys";

interface CandidateOperation {
  execute: () => Promise<void | VoiceLibraryRevision>;
}

export function useVoiceCandidates() {
  const client = useQueryClient();
  const candidatesQuery = useQuery({
    queryKey: VOICE_QUERY_KEYS.candidates,
    queryFn: listVoiceCandidates,
  });
  const runQuery = useQuery({
    queryKey: VOICE_QUERY_KEYS.activeCandidateRun,
    queryFn: getActiveVoiceCandidateRun,
  });
  const activeRunId = shallowRef<string>();
  const localError = shallowRef<string>();
  const isCancelling = shallowRef(false);
  const submission = useVoiceCandidateSubmission();
  let stopObservation: (() => unknown) | undefined;
  let disposed = false;
  const mutation = useMutation({
    mutationFn: ({ execute }: CandidateOperation) => execute(),
    onSettled: () => client.invalidateQueries({ queryKey: VOICE_QUERY_KEYS.candidates }),
    onError: (cause) => {
      if (decodeAppError(cause)?.code !== "cancelled") localError.value = getErrorMessage(cause);
    },
  });
  const candidates = computed(() => (candidatesQuery.data.value ?? []).map(presentVoiceCandidate));
  const busy = computed(() =>
    Boolean(activeRunId.value || submission.jobId.value || mutation.isPending.value || isCancelling.value),
  );
  const errors = computed(() => [
    ...new Set(
      [localError.value, candidatesQuery.error.value, runQuery.error.value]
        .filter((cause): cause is Error | string => Boolean(cause))
        .map((cause) => (typeof cause === "string" ? cause : getErrorMessage(cause))),
    ),
  ]);

  function editCandidates(update: (current: VoiceCandidate[]) => VoiceCandidate[]) {
    client.setQueryData<VoiceCandidate[]>(VOICE_QUERY_KEYS.candidates, (current) => update(current ?? []));
  }

  function observe(run: RunObservation) {
    if (activeRunId.value === run.id) return;
    stopObservation?.();
    activeRunId.value = run.id;
    client.setQueryData(VOICE_QUERY_KEYS.activeCandidateRun, run);
    const settle = () => {
      stopObservation?.();
      stopObservation = undefined;
      activeRunId.value = undefined;
      client.setQueryData(VOICE_QUERY_KEYS.activeCandidateRun, null);
    };
    stopObservation = observeVoiceCandidateRun(
      run,
      (candidate) => {
        settle();
        editCandidates((current) => [
          candidate,
          ...current.filter(({ candidateId }) => candidateId !== candidate.candidateId),
        ]);
      },
      (message) => {
        settle();
        if (message) localError.value = message;
      },
    );
  }

  function operate(execute: () => Promise<void>) {
    if (busy.value) return;
    localError.value = undefined;
    mutation.mutate({ execute });
  }

  function selected(candidateId: string) {
    const candidate = candidatesQuery.data.value?.find((current) => current.candidateId === candidateId);
    if (!candidate) throw new Error("Voice candidate is no longer available");
    return candidate;
  }

  function draft(input: Omit<DraftVoiceCandidateRequest, "requestId">) {
    if (busy.value) return;
    localError.value = undefined;
    void submission
      .start(input)
      .then((run) => !disposed && observe(run))
      .catch((cause: unknown) => {
        if (!disposed && decodeAppError(cause)?.code !== "cancelled")
          localError.value = getErrorMessage(cause);
      });
  }

  function audition(candidateId: string) {
    operate(async () => {
      const updated = await auditionVoiceCandidate(selected(candidateId));
      editCandidates((current) => current.map((item) => (item.candidateId === candidateId ? updated : item)));
    });
  }

  async function save(candidateId: string, name: string) {
    if (busy.value) return;
    localError.value = undefined;
    try {
      return await mutation.mutateAsync({
        execute: async () => {
          const voice = await selectVoiceCandidate(selected(candidateId), name);
          editCandidates((current) => current.filter((candidate) => candidate.candidateId !== candidateId));
          await Promise.all([
            client.invalidateQueries({ queryKey: VOICE_QUERY_KEYS.profiles }),
            client.invalidateQueries({ queryKey: VOICE_QUERY_KEYS.revisions }),
          ]);
          return voice;
        },
      });
    } catch {
      return undefined;
    }
  }

  function discard(candidateId: string) {
    operate(async () => {
      await rejectVoiceCandidate(selected(candidateId));
      editCandidates((current) => current.filter((candidate) => candidate.candidateId !== candidateId));
    });
  }

  async function cancel() {
    const runId = activeRunId.value ?? submission.jobId.value;
    if (!runId || isCancelling.value) return;
    isCancelling.value = true;
    try {
      if (activeRunId.value) await cancelResourceRun(runId);
      else await submission.cancel();
    } catch (cause) {
      if (activeRunId.value === runId || submission.jobId.value === runId)
        localError.value = getErrorMessage(cause);
    } finally {
      isCancelling.value = false;
    }
  }

  watch(runQuery.data, (run) => run && observe(run), { immediate: true });
  onScopeDispose(() => {
    disposed = true;
    stopObservation?.();
  });
  return {
    candidates,
    activeRunId,
    pendingRequestId: submission.jobId,
    busy,
    isCancelling,
    errors,
    draft,
    save,
    cancel,
    discard,
    audition,
  };
}

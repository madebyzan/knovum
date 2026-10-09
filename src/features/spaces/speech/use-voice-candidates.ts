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
import { voiceQueryKeys } from "./voice-query-keys";

interface CandidateOperation {
  execute: () => Promise<void>;
}

export function useVoiceCandidates(created: (voice: VoiceLibraryRevision) => void) {
  const client = useQueryClient();
  const candidatesQuery = useQuery({
    queryKey: voiceQueryKeys.candidates,
    queryFn: listVoiceCandidates,
  });
  const runQuery = useQuery({
    queryKey: voiceQueryKeys.activeCandidateRun,
    queryFn: getActiveVoiceCandidateRun,
  });
  const activeRunId = shallowRef<string>();
  const localError = shallowRef<string>();
  const submission = useVoiceCandidateSubmission();
  let stopObservation: (() => unknown) | undefined;
  let disposed = false;
  const mutation = useMutation({
    mutationFn: ({ execute }: CandidateOperation) => execute(),
    onSettled: () => client.invalidateQueries({ queryKey: voiceQueryKeys.candidates }),
    onError: (cause) => {
      if (decodeAppError(cause)?.code !== "cancelled") localError.value = getErrorMessage(cause);
    },
  });
  const candidates = computed(() => (candidatesQuery.data.value ?? []).map(presentVoiceCandidate));
  const busy = computed(() =>
    Boolean(activeRunId.value || submission.jobId.value || mutation.isPending.value),
  );
  const error = computed(() => {
    const cause = candidatesQuery.error.value ?? runQuery.error.value;
    return localError.value ?? (cause ? getErrorMessage(cause) : undefined);
  });

  function editCandidates(update: (current: VoiceCandidate[]) => VoiceCandidate[]) {
    client.setQueryData<VoiceCandidate[]>(voiceQueryKeys.candidates, (current) => update(current ?? []));
  }

  function observe(run: RunObservation) {
    if (activeRunId.value === run.id) return;
    stopObservation?.();
    activeRunId.value = run.id;
    client.setQueryData(voiceQueryKeys.activeCandidateRun, run);
    const settle = () => {
      stopObservation?.();
      stopObservation = undefined;
      activeRunId.value = undefined;
      client.setQueryData(voiceQueryKeys.activeCandidateRun, null);
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

  function selected(id: string) {
    const candidate = candidatesQuery.data.value?.find(({ candidateId }) => candidateId === id);
    if (!candidate) throw new Error("Voice candidate is no longer available");
    return candidate;
  }

  function draft(input: Omit<DraftVoiceCandidateRequest, "requestId">) {
    if (busy.value) return;
    localError.value = undefined;
    void submission
      .start(input, (run) => !disposed && observe(run))
      .catch((cause: unknown) => {
        if (!disposed && decodeAppError(cause)?.code !== "cancelled")
          localError.value = getErrorMessage(cause);
      });
  }

  function audition(id: string) {
    operate(async () => {
      const updated = await auditionVoiceCandidate(selected(id));
      editCandidates((current) => current.map((item) => (item.candidateId === id ? updated : item)));
    });
  }

  function save(id: string, name: string, authorized: boolean) {
    operate(async () => {
      const voice = await selectVoiceCandidate(selected(id), name, authorized);
      editCandidates((current) => current.filter(({ candidateId }) => candidateId !== id));
      await Promise.all([
        client.invalidateQueries({ queryKey: voiceQueryKeys.profiles }),
        client.invalidateQueries({ queryKey: voiceQueryKeys.revisions }),
      ]);
      created(voice);
    });
  }

  function discard(id: string) {
    operate(async () => {
      await rejectVoiceCandidate(selected(id));
      editCandidates((current) => current.filter(({ candidateId }) => candidateId !== id));
    });
  }

  async function cancel() {
    const runId = activeRunId.value ?? submission.jobId.value;
    if (!runId) return;
    try {
      if (activeRunId.value) await cancelResourceRun(runId);
      else await submission.cancel();
    } catch (cause) {
      if (activeRunId.value === runId || submission.jobId.value === runId)
        localError.value = getErrorMessage(cause);
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
    error,
    draft,
    save,
    cancel,
    discard,
    audition,
  };
}

import { computed, shallowRef } from "vue";

import { i18n } from "../../../foundation/i18n/i18n";
import type { DraftVoiceCandidateRequest } from "../contracts/generated/DraftVoiceCandidateRequest";
import type { RunObservation } from "../contracts/generated/RunObservation";
import { cancelResourceRun, observeResourceRun } from "../runs/resource-run-observation";
import { cancelSpeechGeneration } from "./ipc";
import { draftVoiceCandidate } from "./voice-candidate-ipc";

export function useVoiceCandidateSubmission() {
  const pending = shallowRef<{ id: string; cancelRequested: boolean }>();
  const jobId = computed(() => pending.value?.id);

  async function start(
    input: Omit<DraftVoiceCandidateRequest, "requestId">,
    created: (run: RunObservation) => void,
  ) {
    const job = { id: crypto.randomUUID(), cancelRequested: false };
    pending.value = job;
    try {
      const run = await draftVoiceCandidate({ ...input, requestId: job.id });
      observeResourceRun(run, i18n.global.t("spaces.speech.generateCandidate"));
      created(run);
      if (job.cancelRequested) await cancelResourceRun(run.id);
      return run;
    } finally {
      if (pending.value === job) pending.value = undefined;
    }
  }
  async function cancel() {
    const job = pending.value;
    if (!job) return;
    job.cancelRequested = true;
    await cancelSpeechGeneration(job.id);
  }
  function reset() {
    pending.value = undefined;
  }
  return { jobId, start, cancel, reset };
}

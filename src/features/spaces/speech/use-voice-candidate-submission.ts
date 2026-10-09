import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import type { DraftVoiceCandidateRequest } from "../contracts/generated/DraftVoiceCandidateRequest";
import { cancelResourceRun, observeResourceRun } from "../runs/resource-run-observation";
import { cancelSpeechGeneration } from "./ipc";
import { draftVoiceCandidate } from "./voice-candidate-ipc";

export function useVoiceCandidateSubmission() {
  const { t } = useI18n();
  const pending = shallowRef<{ id: string; cancelRequested: boolean }>();
  const jobId = computed(() => pending.value?.id);

  async function start(input: Omit<DraftVoiceCandidateRequest, "requestId">) {
    const job = { id: crypto.randomUUID(), cancelRequested: false };
    pending.value = job;
    try {
      const run = await draftVoiceCandidate({ ...input, requestId: job.id });
      observeResourceRun(run, t("spaces.speech.voice.candidate.generate"));
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
  return { jobId, start, cancel };
}

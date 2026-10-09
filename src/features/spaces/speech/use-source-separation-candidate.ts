import { computed, onScopeDispose, readonly, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { SpeechMediaSource } from "../contracts/generated/SpeechMediaSource";
import type { SpeechSourceSeparationCandidate } from "../contracts/generated/SpeechSourceSeparationCandidate";
import { cancelResourceRun, observeResourceRun } from "../runs/resource-run-observation";
import { workflowStateLabelKey } from "../workflows/workflow-state-label";
import {
  getLatestSourceSeparationRun,
  getSpeechSourceSeparation,
  reviewSpeechSourceSeparation,
  startSpeechSourceSeparation,
} from "./source-separation-ipc";

const TERMINAL_STATES = new Set(["succeeded", "completed_with_errors", "failed", "cancelled"]);

export function useSourceSeparationCandidate() {
  const { t } = useI18n();
  const candidate = shallowRef<SpeechSourceSeparationCandidate>();
  const activeRunId = shallowRef<string>();
  const preparing = shallowRef(false);
  const loading = shallowRef(false);
  const reviewing = shallowRef(false);
  const cancelling = shallowRef(false);
  const busy = computed(() => preparing.value || loading.value || reviewing.value || cancelling.value);
  const error = shallowRef<string>();
  let stopObservation: (() => unknown) | undefined;
  let epoch = 0;

  function reset() {
    epoch += 1;
    stopObservation?.();
    stopObservation = undefined;
    candidate.value = undefined;
    activeRunId.value = undefined;
    preparing.value = false;
    loading.value = false;
    reviewing.value = false;
    cancelling.value = false;
    error.value = undefined;
  }

  async function restore(spaceId: string, source: SpeechMediaSource) {
    reset();
    const current = epoch;
    preparing.value = true;
    try {
      const run = await getLatestSourceSeparationRun(spaceId, source);
      if (run && current === epoch) {
        activeRunId.value = run.id;
        await observe(run, current);
      }
    } catch (cause) {
      if (current === epoch) error.value = getErrorMessage(cause);
    } finally {
      if (current === epoch) preparing.value = false;
    }
  }

  async function start(spaceId: string, source: SpeechMediaSource) {
    if (busy.value || activeRunId.value) return;
    const requestId = crypto.randomUUID();
    const current = ++epoch;
    candidate.value = undefined;
    error.value = undefined;
    preparing.value = true;
    try {
      const run = await startSpeechSourceSeparation({ requestId, spaceId, source });
      if (current !== epoch) return;
      activeRunId.value = run.id;
      await observe(run, current);
    } catch (cause) {
      if (current === epoch) error.value = getErrorMessage(cause);
    } finally {
      if (current === epoch) preparing.value = false;
    }
  }

  async function observe(run: RunObservation, current: number) {
    stopObservation?.();
    if (TERMINAL_STATES.has(run.state)) {
      activeRunId.value = undefined;
      if (run.state === "succeeded") await load(run.id, current);
      else error.value = run.state === "cancelled" ? undefined : t(workflowStateLabelKey(run.state));
      return;
    }
    stopObservation = observeResourceRun(run, t("spaces.speech.separation.task"), (next) => {
      if (current !== epoch || !TERMINAL_STATES.has(next.state)) return;
      stopObservation?.();
      stopObservation = undefined;
      activeRunId.value = undefined;
      if (next.state === "succeeded") void load(run.id, current);
      else error.value = next.state === "cancelled" ? undefined : t(workflowStateLabelKey(next.state));
    });
  }

  async function load(separationId: string, current: number) {
    loading.value = true;
    try {
      const loaded = await getSpeechSourceSeparation(separationId);
      if (current === epoch) candidate.value = loaded;
    } catch (cause) {
      if (current === epoch) error.value = getErrorMessage(cause);
    } finally {
      if (current === epoch) loading.value = false;
    }
  }

  async function cancel() {
    const runId = activeRunId.value;
    const current = epoch;
    if (!runId || cancelling.value) return;
    cancelling.value = true;
    try {
      await cancelResourceRun(runId);
    } catch (cause) {
      if (current === epoch && activeRunId.value === runId) error.value = getErrorMessage(cause);
    } finally {
      if (current === epoch) cancelling.value = false;
    }
  }

  async function review(accepted: boolean) {
    const current = candidate.value;
    if (!current || current.reviewStatus !== "pending" || busy.value) return;
    const reviewEpoch = epoch;
    reviewing.value = true;
    try {
      const reviewed = await reviewSpeechSourceSeparation({
        separationId: current.separationId,
        expectedReviewStatus: "pending",
        accepted,
      });
      if (reviewEpoch !== epoch) return;
      candidate.value = reviewed;
      error.value = undefined;
    } catch (cause) {
      if (reviewEpoch === epoch) error.value = getErrorMessage(cause);
    } finally {
      if (reviewEpoch === epoch) reviewing.value = false;
    }
  }

  onScopeDispose(reset);
  return {
    restore,
    candidate: readonly(candidate),
    activeRunId: readonly(activeRunId),
    busy: readonly(busy),
    error: readonly(error),
    start,
    cancel,
    review,
    reset,
  };
}

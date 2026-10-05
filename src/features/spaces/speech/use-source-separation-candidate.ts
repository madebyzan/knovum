import { onScopeDispose, readonly, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/i18n";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { SpeechMediaSource } from "../contracts/generated/SpeechMediaSource";
import type { SpeechSourceSeparationCandidate } from "../contracts/generated/SpeechSourceSeparationCandidate";
import { cancelResourceRun, observeResourceRun } from "../runs/resource-run-observation";
import { workflowStateLabelKey } from "../workflows/workflow-state-label";
import { getSpeechSourceSeparation, reviewSpeechSourceSeparation, startSpeechSourceSeparation } from "./ipc";

const TERMINAL_STATES = new Set(["succeeded", "completed_with_errors", "failed", "cancelled"]);

export function useSourceSeparationCandidate() {
  const candidate = shallowRef<SpeechSourceSeparationCandidate>();
  const activeRunId = shallowRef<string>();
  const busy = shallowRef(false);
  const error = shallowRef<string>();
  let stopObservation: (() => unknown) | undefined;
  let epoch = 0;

  function reset() {
    epoch += 1;
    stopObservation?.();
    stopObservation = undefined;
    candidate.value = undefined;
    activeRunId.value = undefined;
    busy.value = false;
    error.value = undefined;
  }

  async function start(spaceId: string, source: SpeechMediaSource) {
    if (busy.value || activeRunId.value) return;
    const requestId = crypto.randomUUID();
    const current = ++epoch;
    candidate.value = undefined;
    error.value = undefined;
    busy.value = true;
    try {
      const run = await startSpeechSourceSeparation({ requestId, spaceId, source });
      if (current !== epoch) return;
      activeRunId.value = run.id;
      observe(run, current);
    } catch (cause) {
      if (current === epoch) error.value = getErrorMessage(cause);
    } finally {
      if (current === epoch) busy.value = false;
    }
  }

  function observe(run: RunObservation, current: number) {
    stopObservation?.();
    if (TERMINAL_STATES.has(run.state)) {
      activeRunId.value = undefined;
      if (run.state === "succeeded") void load(run.id, current);
      else error.value = i18n.global.t(workflowStateLabelKey(run.state));
      return;
    }
    stopObservation = observeResourceRun(run, i18n.global.t("spaces.speech.sourceSeparationTask"), (next) => {
      if (current !== epoch || !TERMINAL_STATES.has(next.state)) return;
      stopObservation?.();
      stopObservation = undefined;
      activeRunId.value = undefined;
      if (next.state === "succeeded") void load(run.id, current);
      else error.value = i18n.global.t(workflowStateLabelKey(next.state));
    });
  }

  async function load(separationId: string, current: number) {
    try {
      const loaded = await getSpeechSourceSeparation(separationId);
      if (current === epoch) candidate.value = loaded;
    } catch (cause) {
      if (current === epoch) error.value = getErrorMessage(cause);
    }
  }

  async function cancel() {
    if (activeRunId.value) await cancelResourceRun(activeRunId.value);
  }

  async function review(accepted: boolean) {
    const current = candidate.value;
    if (!current || current.reviewStatus !== "pending" || busy.value) return;
    busy.value = true;
    try {
      candidate.value = await reviewSpeechSourceSeparation({
        separationId: current.separationId,
        expectedReviewStatus: "pending",
        accepted,
      });
      error.value = undefined;
    } catch (cause) {
      error.value = getErrorMessage(cause);
    } finally {
      busy.value = false;
    }
  }

  onScopeDispose(reset);
  return {
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

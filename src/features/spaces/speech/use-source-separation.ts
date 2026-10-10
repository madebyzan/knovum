import { computed, onScopeDispose, readonly, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { SpeechMediaSource } from "../contracts/generated/SpeechMediaSource";
import { cancelResourceRun, observeResourceRun } from "../runs/resource-run-observation";
import { workflowStateLabelKey } from "../workflows/workflow-state-label";
import { getLatestSourceSeparationRun, startSpeechSourceSeparation } from "./source-separation-ipc";

const TERMINAL_STATES = new Set(["succeeded", "completed_with_errors", "failed", "cancelled"]);

export function useSourceSeparation() {
  const { t } = useI18n();
  const activeRunId = shallowRef<string>();
  const runState = shallowRef<RunObservation["state"]>();
  const isPreparing = shallowRef(false);
  const isCancelling = shallowRef(false);
  const isBusy = computed(() => isPreparing.value || isCancelling.value);
  const error = shallowRef<string>();
  let stopObservation: (() => unknown) | undefined;
  let observationEpoch = 0;

  function reset() {
    observationEpoch += 1;
    stopObservation?.();
    stopObservation = undefined;
    activeRunId.value = undefined;
    runState.value = undefined;
    isPreparing.value = false;
    isCancelling.value = false;
    error.value = undefined;
  }

  function settle(run: RunObservation) {
    runState.value = run.state;
    activeRunId.value = undefined;
    error.value = ["succeeded", "cancelled"].includes(run.state)
      ? undefined
      : t(workflowStateLabelKey(run.state));
  }

  function observe(run: RunObservation, currentEpoch: number) {
    runState.value = run.state;
    if (TERMINAL_STATES.has(run.state)) {
      settle(run);
      return;
    }
    activeRunId.value = run.id;
    stopObservation = observeResourceRun(run, t("spaces.speech.separation.task"), (next) => {
      if (currentEpoch !== observationEpoch) return;
      runState.value = next.state;
      if (!TERMINAL_STATES.has(next.state)) return;
      stopObservation?.();
      stopObservation = undefined;
      settle(next);
    });
  }

  async function restore(spaceId: string, sourceNodeId: string, source: SpeechMediaSource) {
    reset();
    const currentEpoch = observationEpoch;
    isPreparing.value = true;
    try {
      const run = await getLatestSourceSeparationRun(spaceId, sourceNodeId, source);
      if (run && currentEpoch === observationEpoch) observe(run, currentEpoch);
    } catch (cause) {
      if (currentEpoch === observationEpoch) error.value = getErrorMessage(cause);
    } finally {
      if (currentEpoch === observationEpoch) isPreparing.value = false;
    }
  }

  async function start(spaceId: string, sourceNodeId: string, source: SpeechMediaSource) {
    if (isBusy.value || activeRunId.value) return;
    const currentEpoch = ++observationEpoch;
    isPreparing.value = true;
    error.value = undefined;
    runState.value = undefined;
    try {
      const run = await startSpeechSourceSeparation({
        requestId: crypto.randomUUID(),
        spaceId,
        sourceNodeId,
        source,
        vocalsLabel: t("spaces.speech.separation.stems.vocals"),
        backgroundLabel: t("spaces.speech.separation.stems.background"),
      });
      // The shared observer owns task lifetime and refreshes published nodes even after selection changes.
      if (currentEpoch !== observationEpoch || TERMINAL_STATES.has(run.state))
        observeResourceRun(run, t("spaces.speech.separation.task"));
      if (currentEpoch === observationEpoch) observe(run, currentEpoch);
    } catch (cause) {
      if (currentEpoch === observationEpoch) error.value = getErrorMessage(cause);
    } finally {
      if (currentEpoch === observationEpoch) isPreparing.value = false;
    }
  }

  async function cancel() {
    const runId = activeRunId.value;
    const currentEpoch = observationEpoch;
    if (!runId || isCancelling.value) return;
    isCancelling.value = true;
    try {
      await cancelResourceRun(runId);
    } catch (cause) {
      if (currentEpoch === observationEpoch && activeRunId.value === runId)
        error.value = getErrorMessage(cause);
    } finally {
      if (currentEpoch === observationEpoch) isCancelling.value = false;
    }
  }

  onScopeDispose(reset);
  return {
    restore,
    activeRunId: readonly(activeRunId),
    isBusy: readonly(isBusy),
    isPreparing: readonly(isPreparing),
    isCancelling: readonly(isCancelling),
    runState: readonly(runState),
    error: readonly(error),
    start,
    cancel,
    reset,
  };
}

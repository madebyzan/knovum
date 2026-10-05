import { onScopeDispose, readonly, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/i18n";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { TranslateVideoDubbingRequest } from "../contracts/generated/TranslateVideoDubbingRequest";
import type { VideoTranslation } from "../contracts/generated/VideoTranslation";
import { cancelResourceRun, observeResourceRun } from "../runs/resource-run-observation";
import { workflowStateLabelKey } from "../workflows/workflow-state-label";
import { getVideoTranslation, getVideoTranslationRun, translateVideoDubbing } from "./ipc";

const TERMINAL_STATES = new Set(["succeeded", "completed_with_errors", "failed", "cancelled"]);

export function useVideoTranslationRun(
  apply: (translation: VideoTranslation) => void,
  report: (message: string) => void,
) {
  const activeRunId = shallowRef<string>();
  const submitting = shallowRef(false);
  let stopObservation: (() => unknown) | undefined;
  let epoch = 0;

  function reset() {
    epoch += 1;
    stopObservation?.();
    stopObservation = undefined;
    activeRunId.value = undefined;
    submitting.value = false;
  }

  async function restore(projectId: string, expectedRevision: number) {
    const current = ++epoch;
    stopObservation?.();
    stopObservation = undefined;
    activeRunId.value = undefined;
    try {
      const run = await getVideoTranslationRun(projectId, expectedRevision);
      if (current === epoch && run) attach(run, current);
    } catch (cause) {
      if (current === epoch) report(getErrorMessage(cause));
    }
  }

  async function start(request: TranslateVideoDubbingRequest) {
    if (submitting.value || activeRunId.value) return;
    const current = ++epoch;
    submitting.value = true;
    try {
      const run = await translateVideoDubbing(request);
      if (current === epoch) attach(run, current);
    } catch (cause) {
      if (current === epoch) report(getErrorMessage(cause));
    } finally {
      if (current === epoch) submitting.value = false;
    }
  }

  function attach(run: RunObservation, current: number) {
    stopObservation?.();
    if (TERMINAL_STATES.has(run.state)) {
      activeRunId.value = undefined;
      if (run.state === "succeeded") void load(run.id, current);
      else report(i18n.global.t(workflowStateLabelKey(run.state)));
      return;
    }
    activeRunId.value = run.id;
    stopObservation = observeResourceRun(run, i18n.global.t("spaces.speech.translateScript"), (next) => {
      if (current !== epoch || !TERMINAL_STATES.has(next.state)) return;
      stopObservation?.();
      stopObservation = undefined;
      activeRunId.value = undefined;
      if (next.state === "succeeded") void load(next.id, current);
      else report(i18n.global.t(workflowStateLabelKey(next.state)));
    });
  }

  async function load(runId: string, current: number) {
    try {
      const translation = await getVideoTranslation(runId);
      if (current === epoch) apply(translation);
    } catch (cause) {
      if (current === epoch) report(getErrorMessage(cause));
    }
  }

  async function cancel() {
    if (activeRunId.value) await cancelResourceRun(activeRunId.value);
  }

  onScopeDispose(reset);
  return {
    activeRunId: readonly(activeRunId),
    submitting: readonly(submitting),
    restore,
    start,
    cancel,
  };
}

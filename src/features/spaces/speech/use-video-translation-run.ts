import { onScopeDispose, readonly, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { TranslateVideoDubbingRequest } from "../contracts/generated/TranslateVideoDubbingRequest";
import type { VideoTranslation } from "../contracts/generated/VideoTranslation";
import { cancelResourceRun, observeResourceRun } from "../runs/resource-run-observation";
import { runStateLabelKey } from "../runs/run-state-label";
import { getVideoTranslation, getVideoTranslationRun, translateVideoDubbing } from "./ipc";

const TERMINAL_STATES = new Set(["succeeded", "completed_with_errors", "failed", "cancelled"]);

export function useVideoTranslationRun(translated: (translation: VideoTranslation) => void) {
  const { t } = useI18n();
  const activeRunId = shallowRef<string>();
  const submitting = shallowRef(false);
  const cancelling = shallowRef(false);
  const error = shallowRef<string>();
  let stopObservation: (() => unknown) | undefined;
  let epoch = 0;

  function reset() {
    epoch += 1;
    stopObservation?.();
    stopObservation = undefined;
    activeRunId.value = undefined;
    submitting.value = false;
    cancelling.value = false;
    error.value = undefined;
  }

  async function restore(projectId: string, expectedRevision: number) {
    reset();
    const current = epoch;
    submitting.value = true;
    try {
      const run = await getVideoTranslationRun(projectId, expectedRevision);
      if (current === epoch && run) await attach(run, current);
    } catch (cause) {
      if (current === epoch) error.value = getErrorMessage(cause);
    } finally {
      if (current === epoch) submitting.value = false;
    }
  }

  async function start(request: TranslateVideoDubbingRequest) {
    if (submitting.value || activeRunId.value) return;
    const current = ++epoch;
    submitting.value = true;
    error.value = undefined;
    try {
      const run = await translateVideoDubbing(request);
      if (current === epoch) await attach(run, current);
    } catch (cause) {
      if (current === epoch) error.value = getErrorMessage(cause);
    } finally {
      if (current === epoch) submitting.value = false;
    }
  }

  async function attach(run: RunObservation, current: number) {
    stopObservation?.();
    activeRunId.value = run.id;
    if (TERMINAL_STATES.has(run.state)) {
      if (run.state === "succeeded") await load(run.id, current);
      else if (run.state === "cancelled") {
        activeRunId.value = undefined;
      } else {
        activeRunId.value = undefined;
        error.value = t(runStateLabelKey(run.state));
      }
      return;
    }
    stopObservation = observeResourceRun(run, t("spaces.speech.translation.translate"), (next) => {
      if (current !== epoch || !TERMINAL_STATES.has(next.state)) return;
      stopObservation?.();
      stopObservation = undefined;
      if (next.state === "succeeded") void load(next.id, current);
      else if (next.state === "cancelled") {
        activeRunId.value = undefined;
      } else {
        activeRunId.value = undefined;
        error.value = t(runStateLabelKey(next.state));
      }
    });
  }

  async function load(runId: string, current: number) {
    activeRunId.value = undefined;
    submitting.value = true;
    try {
      const translation = await getVideoTranslation(runId);
      if (current === epoch) {
        error.value = undefined;
        translated(translation);
      }
    } catch (cause) {
      if (current === epoch) error.value = getErrorMessage(cause);
    } finally {
      if (current === epoch) submitting.value = false;
    }
  }

  async function cancel() {
    const runId = activeRunId.value;
    const current = epoch;
    if (!runId || cancelling.value) return;
    cancelling.value = true;
    error.value = undefined;
    try {
      await cancelResourceRun(runId);
    } catch (cause) {
      if (current === epoch && activeRunId.value === runId) error.value = getErrorMessage(cause);
    } finally {
      if (current === epoch) cancelling.value = false;
    }
  }

  onScopeDispose(reset);
  return {
    activeRunId: readonly(activeRunId),
    submitting: readonly(submitting),
    cancelling: readonly(cancelling),
    error: readonly(error),
    restore,
    start,
    cancel,
  };
}

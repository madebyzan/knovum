import { error as logError } from "@tauri-apps/plugin-log";
import { onScopeDispose, readonly, shallowRef, watch, type Ref } from "vue";

import { formatErrorDiagnostic, getErrorMessage } from "../../../foundation/errors/error-format";
import type { SpeechSignalReport } from "../contracts/generated/SpeechSignalReport";
import { cancelSpeechGeneration } from "./ipc";
import { inspectSpeechSignal } from "./signal-inspection-ipc";

interface InspectionContext {
  projectId: Readonly<Ref<string>>;
  revisionId: Readonly<Ref<string>>;
}

export function useSpeechSignalInspection(context: InspectionContext) {
  const report = shallowRef<SpeechSignalReport>();
  const error = shallowRef<string>();
  const jobId = shallowRef<string>();
  const scannedMs = shallowRef(0);
  let epoch = 0;
  let disposed = false;

  async function cancelJob(job: string, expectedEpoch: number, reportFailure: boolean) {
    try {
      await cancelSpeechGeneration(job);
    } catch (cause) {
      if (reportFailure && !disposed && epoch === expectedEpoch) error.value = getErrorMessage(cause);
      else void logError(`Speech signal inspection cancellation failed: ${formatErrorDiagnostic(cause)}`);
    }
  }

  function detachJob(reportFailure: boolean) {
    const job = jobId.value;
    jobId.value = undefined;
    const cancellationEpoch = ++epoch;
    return job ? cancelJob(job, cancellationEpoch, reportFailure) : Promise.resolve();
  }

  function reset() {
    void detachJob(false);
    report.value = undefined;
    scannedMs.value = 0;
    error.value = undefined;
  }

  async function inspect(): Promise<SpeechSignalReport | undefined> {
    if (jobId.value) return undefined;
    const version = ++epoch;
    const job = crypto.randomUUID();
    jobId.value = job;
    error.value = undefined;
    report.value = undefined;
    scannedMs.value = 0;
    try {
      const inspection = await inspectSpeechSignal(
        { projectId: context.projectId.value, revisionId: context.revisionId.value, jobId: job },
        (value) => {
          if (version === epoch) scannedMs.value = value;
        },
      );
      if (version !== epoch) return undefined;
      report.value = inspection;
      return inspection;
    } catch (cause) {
      if (version === epoch) error.value = getErrorMessage(cause);
      return undefined;
    } finally {
      if (version === epoch) jobId.value = undefined;
    }
  }

  watch([context.projectId, context.revisionId], reset, { flush: "sync" });
  onScopeDispose(() => {
    disposed = true;
    void detachJob(false);
  });
  return {
    cancel: () => detachJob(true),
    error: readonly(error),
    inspect,
    jobId: readonly(jobId),
    report: readonly(report),
    scannedMs: readonly(scannedMs),
  };
}

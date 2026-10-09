import { error as logError } from "@tauri-apps/plugin-log";
import { computed, onScopeDispose, readonly, shallowRef, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import { formatErrorDiagnostic, getErrorMessage } from "../../../foundation/errors/error-format";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { ImportedAudiobookScript } from "./audiobook-script";
import { readNodeAudiobookScript } from "./audiobook-script-import";
import { cancelSpeechGeneration } from "./ipc";

export function useAudiobookScriptImport(node: Readonly<Ref<SpaceCanvasNode>>) {
  const { t } = useI18n();
  const error = shallowRef<string>();
  const jobId = shallowRef<string>();
  const script = shallowRef<ImportedAudiobookScript>();
  let epoch = 0;
  let disposed = false;

  async function cancelJob(job: string, expectedEpoch: number, reportFailure: boolean) {
    try {
      await cancelSpeechGeneration(job);
    } catch (cause) {
      if (reportFailure && !disposed && epoch === expectedEpoch) error.value = getErrorMessage(cause);
      else void logError(`Audiobook script import cancellation failed: ${formatErrorDiagnostic(cause)}`);
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
    script.value = undefined;
    error.value = undefined;
  }

  async function importScript(): Promise<ImportedAudiobookScript | undefined> {
    if (jobId.value) return undefined;
    const version = ++epoch;
    const job = crypto.randomUUID();
    jobId.value = job;
    error.value = undefined;
    script.value = undefined;
    try {
      const imported = await readNodeAudiobookScript(node.value, t("spaces.speech.project.defaultRole"), job);
      if (version !== epoch) return undefined;
      script.value = imported;
      return imported;
    } catch (cause) {
      if (version === epoch) error.value = getErrorMessage(cause);
      return undefined;
    } finally {
      if (version === epoch) jobId.value = undefined;
    }
  }

  watch(
    [() => node.value.id, () => node.value.data.sourceRevision, () => node.value.data.knowledgeRevision],
    reset,
    { flush: "sync" },
  );
  onScopeDispose(() => {
    disposed = true;
    void detachJob(false);
  });
  return {
    cancel: () => detachJob(true),
    error: readonly(error),
    importing: computed(() => Boolean(jobId.value)),
    importScript,
    script: readonly(script),
  };
}

import { computed, onScopeDispose, shallowRef } from "vue";
import { toast } from "vue-sonner";

import { cancelAiJob } from "../../../foundation/ai/ai-runtime";
import { prepareNarrationScriptStream } from "../../../foundation/ai/narration-preparation";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { NarrationPreparationMode } from "../../../shared/contracts/generated/NarrationPreparationMode";
import type { TaskTarget } from "../../../shared/contracts/generated/TaskTarget";
import { sanitizeSpokenText } from "./speech-text-sanitizer";

function isAbortException(error: unknown) {
  return error instanceof DOMException && error.name === "AbortError";
}

/**
 * Coordinates user-triggered Knovum narration preparation (Clean / Adapt) with live streaming,
 * cancellation, post-sanitization, and single-click restore to the pre-edit script snapshot.
 */
export function useNarrationScriptPreparation<Snapshot = string>() {
  const preparing = shallowRef(false);
  const previousSnapshot = shallowRef<Snapshot>();
  let activeJobId: string | undefined;
  let controller: AbortController | undefined;

  function cancel() {
    const jobId = activeJobId;
    controller?.abort();
    controller = undefined;
    activeJobId = undefined;
    preparing.value = false;
    if (jobId) void cancelAiJob(jobId).catch(() => {});
  }

  async function run(options: {
    mode: NarrationPreparationMode;
    sourceText: string;
    snapshot: Snapshot;
    target?: TaskTarget | undefined;
    onPreview?: (partial: string) => void;
    onComplete: (cleaned: string) => void;
    onRestore: (snapshot: Snapshot) => void;
  }): Promise<boolean> {
    if (preparing.value || !options.sourceText.trim()) return false;
    cancel();
    const jobId = crypto.randomUUID();
    const abort = new AbortController();
    activeJobId = jobId;
    controller = abort;
    preparing.value = true;
    let streamed = "";
    try {
      const completed = await prepareNarrationScriptStream({
        mode: options.mode,
        text: options.sourceText,
        jobId,
        target: options.target,
        signal: abort.signal,
        onEvent: (event) => {
          if (abort.signal.aborted || event.event !== "delta") return;
          streamed += event.text;
          options.onPreview?.(streamed);
        },
      });
      if (abort.signal.aborted) {
        options.onRestore(options.snapshot);
        return false;
      }
      const sanitized = sanitizeSpokenText(completed || streamed);
      if (!sanitized) {
        options.onRestore(options.snapshot);
        return false;
      }
      previousSnapshot.value = options.snapshot;
      options.onComplete(sanitized);
      return true;
    } catch (error) {
      options.onRestore(options.snapshot);
      if (!abort.signal.aborted && !isAbortException(error)) {
        toast.error(getErrorMessage(error));
      }
      return false;
    } finally {
      if (activeJobId === jobId) {
        activeJobId = undefined;
        controller = undefined;
        preparing.value = false;
      }
    }
  }

  function restore(apply: (snapshot: Snapshot) => void) {
    if (previousSnapshot.value === undefined) return;
    apply(previousSnapshot.value);
    previousSnapshot.value = undefined;
  }

  function clearHistory() {
    previousSnapshot.value = undefined;
  }

  onScopeDispose(cancel);

  return {
    preparing,
    canRestore: computed(() => previousSnapshot.value !== undefined && !preparing.value),
    run,
    cancel,
    restore,
    clearHistory,
  };
}

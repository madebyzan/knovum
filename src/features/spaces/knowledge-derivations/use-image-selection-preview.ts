import { debounce } from "es-toolkit";
import { onScopeDispose, shallowRef } from "vue";
import { toast } from "vue-sonner";

import { cancelAiJob } from "../../../foundation/ai/ai-runtime";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import { previewImageSelection, type SelectionPreviewInput } from "./image-selection-preview";
function hasForeground(selection: SelectionPreviewInput["selection"]) {
  return selection.boxRect || selection.points.some((point) => point.foreground);
}
export function useImageSelectionPreview(input: () => SelectionPreviewInput | undefined) {
  const maskSource = shallowRef<string>();
  const busy = shallowRef(false);
  const error = shallowRef<string>();
  let previewJobId: string | undefined;
  let revision = 0;
  function isCurrent(jobId: string, version: number) {
    return previewJobId === jobId && revision === version;
  }
  function stop() {
    revision++;
    const previous = previewJobId;
    previewJobId = undefined;
    if (previous) void cancelAiJob(previous).catch((cause: unknown) => toast.error(getErrorMessage(cause)));
    busy.value = false;
  }
  async function refresh() {
    if (previewJobId) return;
    const version = revision;
    const current = input();
    if (!current || !hasForeground(current.selection)) {
      busy.value = false;
      return;
    }
    const jobId = crypto.randomUUID();
    previewJobId = jobId;
    try {
      const mask = await previewImageSelection({ ...current, jobId });
      if (isCurrent(jobId, version)) maskSource.value = mask;
    } catch (cause) {
      if (isCurrent(jobId, version)) error.value = getErrorMessage(cause);
    } finally {
      if (previewJobId === jobId) {
        previewJobId = undefined;
        if (revision !== version) {
          schedule.cancel();
          void refresh();
        } else busy.value = false;
      }
    }
  }
  const schedule = debounce(() => void refresh(), 150);
  function queue() {
    revision++;
    maskSource.value = undefined;
    error.value = undefined;
    busy.value = true;
    schedule();
  }
  function clear() {
    schedule.cancel();
    stop();
    maskSource.value = undefined;
    error.value = undefined;
  }
  onScopeDispose(clear);
  return { maskSource, busy, error, queue, clear };
}

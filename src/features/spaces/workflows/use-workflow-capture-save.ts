import { shallowRef } from "vue";

import {
  saveCapturedWorkflow,
  type SavedCapturedWorkflow,
  type WorkflowCapturePreview,
} from "./captured-workflow-ipc";

export function useWorkflowCaptureSave() {
  const name = shallowRef("");
  const saving = shallowRef(false);
  const saved = shallowRef<SavedCapturedWorkflow>();
  const error = shallowRef("");
  let requestId: string | undefined;
  let requestSignature: string | undefined;
  let generation = 0;

  function reset() {
    generation++;
    requestId = undefined;
    requestSignature = undefined;
    saving.value = false;
    saved.value = undefined;
    error.value = "";
  }

  function idFor(signature: string) {
    if (!requestId || requestSignature !== signature) {
      requestId = crypto.randomUUID();
      requestSignature = signature;
    }
    return requestId;
  }

  async function submit(
    spaceId: string,
    selectedNodeIds: string[],
    preview: WorkflowCapturePreview,
    outputNames: Record<string, string>,
  ) {
    if (saving.value || !preview.candidate || !preview.sourceDigest || !name.value.trim()) return;
    const normalizedOutputNames = Object.fromEntries(
      Object.entries(outputNames).map(([id, value]) => [id, value.trim()]),
    );
    const signature = JSON.stringify([
      spaceId,
      selectedNodeIds,
      preview.sourceDigest,
      name.value.trim(),
      normalizedOutputNames,
    ]);
    const currentRequestId = idFor(signature);
    const current = generation;
    saving.value = true;
    error.value = "";
    try {
      const savedWorkflow = await saveCapturedWorkflow({
        requestId: currentRequestId,
        spaceId,
        selectedNodeIds: [...selectedNodeIds],
        sourceDigest: preview.sourceDigest,
        name: name.value.trim(),
        outputNames: normalizedOutputNames,
      });
      if (generation === current) saved.value = savedWorkflow;
    } catch (cause) {
      if (generation === current) error.value = String(cause);
    } finally {
      if (generation === current) saving.value = false;
    }
  }

  return { name, saving, saved, error, reset, submit };
}

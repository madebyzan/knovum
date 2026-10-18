import { computed, onScopeDispose, readonly, shallowRef, type DeepReadonly } from "vue";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { SpeechProjectSnapshot } from "../contracts/generated/SpeechProjectSnapshot";
import { observeResourceRun } from "../runs/resource-run-observation";
import { useSpaceStore } from "../session/space-store";
import { cancelSpeechGeneration, exportSpeechRender, generateSpeechProject } from "./ipc";
import { useSpeechPreview } from "./use-speech-preview";

interface SubmissionContext {
  epoch: number;
  errorEpoch: number;
  projectId: string;
  spaceId: string;
}

/**
 * Composing speech and running it are owned by different places. Auditioning a line is a short job
 * that belongs to the editor, because its result is heard there. Generating the deliverable is a
 * Run: the Task Center carries its progress, its cancellation and its outcome, and the editor's
 * part ends the moment the Run exists.
 */
export function useSpeechGeneration(owner: {
  project: () => DeepReadonly<SpeechProjectSnapshot> | undefined;
}) {
  const store = useSpaceStore();
  const {
    preview,
    progress,
    jobId,
    listening,
    error: previewError,
    previewSelection,
    clearPreview,
    reset: resetPreview,
    cancel: cancelPreview,
    stopListening,
  } = useSpeechPreview();
  const submitting = shallowRef(false);
  const preparingRunId = shallowRef<string>();
  const cancelling = shallowRef(false);
  const operationError = shallowRef<string>();
  const errors = computed(() =>
    [operationError.value, previewError.value].filter((message): message is string => Boolean(message)),
  );
  /** A job the editor itself is running: an audition, or the preparation before a Run exists. */
  const previewing = computed(() => Boolean(jobId.value ?? preparingRunId.value));
  /** A Run the Task Center is carrying for this project. */
  const generating = computed(() => Boolean(owner.project()?.project.activeRunId));
  let submissionEpoch = 0;
  let operationEpoch = 0;
  let cancellationEpoch = 0;

  function reset() {
    resetPreview();
    submissionEpoch += 1;
    operationEpoch += 1;
    cancellationEpoch += 1;
    submitting.value = false;
    preparingRunId.value = undefined;
    cancelling.value = false;
    operationError.value = undefined;
  }

  function isCurrentSubmission(context: SubmissionContext) {
    return (
      context.epoch === submissionEpoch &&
      store.activeSpaceId === context.spaceId &&
      owner.project()?.project.id === context.projectId
    );
  }
  function reportSubmissionError(cause: unknown, context: SubmissionContext) {
    if (
      decodeAppError(cause)?.code !== "cancelled" &&
      context.errorEpoch === operationEpoch &&
      isCurrentSubmission(context)
    )
      operationError.value = getErrorMessage(cause);
  }
  /** Resolves to whether a Run was handed over, so the editor can step out of the way. */
  async function generateProject() {
    const project = owner.project()?.project;
    const spaceId = store.activeSpaceId;
    if (!project || !spaceId || previewing.value || generating.value || submitting.value) return false;
    const context = {
      epoch: ++submissionEpoch,
      errorEpoch: ++operationEpoch,
      projectId: project.id,
      spaceId,
    };
    submitting.value = true;
    operationError.value = undefined;
    const requestId = crypto.randomUUID();
    preparingRunId.value = requestId;
    try {
      const run = await generateSpeechProject({
        requestId,
        projectId: project.id,
        expectedRevision: project.activeRevision,
      });
      observeResourceRun(run, project.title);
      return true;
    } catch (cause) {
      reportSubmissionError(cause, context);
      return false;
    } finally {
      if (context.epoch === submissionEpoch) {
        submitting.value = false;
        preparingRunId.value = undefined;
      }
    }
  }

  /** Cancels the editor's own job. A Run is cancelled where it is reported: the Task Center. */
  async function cancelPreviewJob() {
    if (cancelling.value) return;
    const epoch = ++operationEpoch;
    const cancelEpoch = ++cancellationEpoch;
    cancelling.value = true;
    operationError.value = undefined;
    try {
      if (jobId.value) await cancelPreview();
      else if (preparingRunId.value) await cancelSpeechGeneration(preparingRunId.value);
      if (epoch === operationEpoch) operationError.value = undefined;
    } catch (cause) {
      if (epoch === operationEpoch) operationError.value = getErrorMessage(cause);
    } finally {
      if (cancelEpoch === cancellationEpoch) cancelling.value = false;
    }
  }

  async function exportRender(path: string) {
    const title = owner.project()?.project.title;
    if (!title) return;
    const epoch = ++operationEpoch;
    operationError.value = undefined;
    try {
      await exportSpeechRender(path, title);
      if (epoch === operationEpoch) operationError.value = undefined;
    } catch (cause) {
      if (epoch === operationEpoch) operationError.value = getErrorMessage(cause);
    }
  }

  onScopeDispose(reset);
  return {
    preview,
    progress,
    listening,
    errors,
    previewing,
    generating,
    submitting: readonly(submitting),
    cancelling: readonly(cancelling),
    reset,
    clearPreview,
    previewSelection,
    generateProject,
    cancelPreviewJob,
    stopListening,
    exportRender,
  };
}

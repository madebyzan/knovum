import { computed, onScopeDispose, readonly, shallowRef } from "vue";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import { cancelResourceRun, observeResourceRun } from "../runs/resource-run-observation";
import { useSpaceStore } from "../session/space-store";
import { cancelSpeechGeneration, exportSpeechRender, generateSpeechProject } from "./ipc";
import { useSpeechPreview } from "./use-speech-preview";
import { useSpeechRunObservation, type SpeechRunOwner } from "./use-speech-run-observation";

interface SubmissionContext {
  epoch: number;
  errorEpoch: number;
  projectId: string;
  spaceId: string;
}

export function useSpeechGeneration(owner: SpeechRunOwner) {
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
  const {
    runState,
    error: observationError,
    attachRun,
    reset: resetObservation,
  } = useSpeechRunObservation(owner);
  const submitting = shallowRef(false);
  const preparingRunId = shallowRef<string>();
  const cancelling = shallowRef(false);
  const operationError = shallowRef<string>();
  const errors = computed(() =>
    [operationError.value, previewError.value, observationError.value].filter((message): message is string =>
      Boolean(message),
    ),
  );
  const hasActiveExecution = computed(() =>
    Boolean(jobId.value ?? preparingRunId.value ?? owner.project()?.project.activeRunId),
  );
  let submissionEpoch = 0;
  let operationEpoch = 0;
  let cancellationEpoch = 0;

  function reset() {
    resetPreview();
    resetObservation();
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
  async function generateProject() {
    const project = owner.project()?.project;
    const spaceId = store.activeSpaceId;
    if (!project || !spaceId || hasActiveExecution.value || submitting.value) return;
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
      if (!isCurrentSubmission(context)) {
        observeResourceRun(run, project.title);
        return;
      }
      attachRun(run, project.id, spaceId);
      await owner.reloadProject(project.id);
      if (context.errorEpoch === operationEpoch) operationError.value = undefined;
    } catch (cause) {
      reportSubmissionError(cause, context);
    } finally {
      if (context.epoch === submissionEpoch) {
        submitting.value = false;
        preparingRunId.value = undefined;
      }
    }
  }

  async function cancelGeneration() {
    if (cancelling.value) return;
    const epoch = ++operationEpoch;
    const cancelEpoch = ++cancellationEpoch;
    cancelling.value = true;
    operationError.value = undefined;
    try {
      if (jobId.value) await cancelPreview();
      else if (preparingRunId.value) await cancelSpeechGeneration(preparingRunId.value);
      else {
        const runId = owner.project()?.project.activeRunId;
        if (runId) await cancelResourceRun(runId);
      }
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
    runState,
    errors,
    hasActiveExecution,
    submitting: readonly(submitting),
    cancelling: readonly(cancelling),
    reset,
    clearPreview,
    previewSelection,
    generateProject,
    cancelGeneration,
    stopListening,
    exportRender,
  };
}

import { computed, onScopeDispose, readonly, shallowRef } from "vue";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import { cancelResourceRun, observeResourceRun } from "../runs/resource-run-observation";
import { useSpaceStore } from "../session/space-store";
import { cancelSpeechGeneration, exportSpeechRender, generateSpeechProject } from "./ipc";
import { useSpeechPreview } from "./use-speech-preview";
import { useSpeechRunObservation, type SpeechRunOwner } from "./use-speech-run-observation";

export function useSpeechGeneration(owner: SpeechRunOwner) {
  const store = useSpaceStore();
  const preview = useSpeechPreview(owner.setError, () => owner.project()?.project.spaceId);
  const submitting = shallowRef(false);
  const preparingJobId = shallowRef<string>();
  const cancelling = shallowRef(false);
  const activeJobId = computed(
    () => preview.jobId.value ?? preparingJobId.value ?? owner.project()?.project.activeRunId,
  );
  const observation = useSpeechRunObservation(owner);
  let submissionEpoch = 0;

  function reset() {
    preview.reset();
    observation.reset();
    submissionEpoch += 1;
    submitting.value = false;
    preparingJobId.value = undefined;
    cancelling.value = false;
  }

  function isCurrentSubmission(epoch: number, projectId: string, spaceId: string) {
    return (
      epoch === submissionEpoch &&
      store.activeSpaceId === spaceId &&
      owner.project()?.project.id === projectId
    );
  }
  function reportSubmissionError(cause: unknown, epoch: number, projectId: string, spaceId: string) {
    if (decodeAppError(cause)?.code === "cancelled") return;
    if (isCurrentSubmission(epoch, projectId, spaceId)) owner.setError(getErrorMessage(cause));
  }
  async function generateProject() {
    const project = owner.project()?.project;
    const spaceId = store.activeSpaceId;
    if (!project || !spaceId || activeJobId.value || submitting.value) return;
    const epoch = ++submissionEpoch;
    submitting.value = true;
    const requestId = crypto.randomUUID();
    preparingJobId.value = requestId;
    try {
      const run = await generateSpeechProject({
        requestId,
        projectId: project.id,
        expectedRevision: project.activeRevision,
      });
      if (!isCurrentSubmission(epoch, project.id, spaceId)) {
        observeResourceRun(run, project.title);
        return;
      }
      observation.attachRun(run, project.id, spaceId);
      await owner.reloadProject(project.id);
      owner.setError();
    } catch (cause) {
      reportSubmissionError(cause, epoch, project.id, spaceId);
    } finally {
      if (epoch === submissionEpoch) {
        submitting.value = false;
        preparingJobId.value = undefined;
      }
    }
  }

  async function cancelGeneration() {
    if (cancelling.value) return;
    cancelling.value = true;
    try {
      if (preview.jobId.value) await preview.cancel();
      else if (preparingJobId.value) await cancelSpeechGeneration(preparingJobId.value);
      else {
        const runId = owner.project()?.project.activeRunId;
        if (runId) await cancelResourceRun(runId);
      }
    } catch (cause) {
      owner.setError(getErrorMessage(cause));
    } finally {
      cancelling.value = false;
    }
  }

  async function exportRender(path: string) {
    const title = owner.project()?.project.title;
    if (!title) return;
    try {
      await exportSpeechRender(path, title);
    } catch (cause) {
      owner.setError(getErrorMessage(cause));
    }
  }

  onScopeDispose(reset);
  return {
    preview: preview.preview,
    progress: preview.progress,
    listening: preview.listening,
    runState: observation.runState,
    activeJobId,
    submitting: readonly(submitting),
    cancelling: readonly(cancelling),
    reset,
    clearPreview: preview.clearPreview,
    previewSelection: preview.previewSelection,
    generateProject,
    cancelGeneration,
    stopListening: preview.stopListening,
    exportRender,
  };
}

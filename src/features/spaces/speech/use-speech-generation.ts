import { computed, onScopeDispose, readonly, shallowRef, watch } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { RunObservation } from "../contracts/generated/RunObservation";
import { cancelResourceRun, observeResourceRun } from "../runs/resource-run-observation";
import { useSpaceStore } from "../session/space-store";
import type { SpeechProjectDetail } from "./contracts";
import { exportSpeechRender, generateSpeechProject } from "./ipc";
import { useSpeechPreview } from "./use-speech-preview";

interface SpeechGenerationOwner {
  project: () => SpeechProjectDetail | undefined;
  reloadProject: (projectId: string) => Promise<void>;
  setError: (message?: string) => void;
}

const TERMINAL_STATES = new Set(["succeeded", "completed_with_errors", "failed", "cancelled"]);

export function useSpeechGeneration(owner: SpeechGenerationOwner) {
  const store = useSpaceStore();
  const preview = useSpeechPreview(owner.setError);
  const runState = shallowRef<RunObservation["state"]>();
  const submitting = shallowRef(false);
  const cancelling = shallowRef(false);
  const activeJobId = computed(() => preview.jobId.value ?? owner.project()?.project.activeRunId);
  let stopObservation: (() => unknown) | undefined;
  let observationEpoch = 0;
  let submissionEpoch = 0;

  function stopRunObservation() {
    observationEpoch += 1;
    stopObservation?.();
    stopObservation = undefined;
    runState.value = undefined;
  }

  function reset() {
    preview.reset();
    stopRunObservation();
    submissionEpoch += 1;
    submitting.value = false;
    cancelling.value = false;
  }

  function attachRun(run: RunObservation, projectId: string, spaceId: string) {
    stopRunObservation();
    const epoch = observationEpoch;
    runState.value = run.state;
    stopObservation = observeResourceRun(run, owner.project()?.project.title ?? "Speech", (next) => {
      if (
        epoch !== observationEpoch ||
        store.activeSpaceId !== spaceId ||
        owner.project()?.project.id !== projectId
      )
        return;
      runState.value = next.state;
      if (TERMINAL_STATES.has(next.state)) {
        void owner.reloadProject(projectId).catch((cause) => owner.setError(getErrorMessage(cause)));
      }
    });
  }

  function isCurrentSubmission(epoch: number, projectId: string, spaceId: string) {
    return (
      epoch === submissionEpoch &&
      store.activeSpaceId === spaceId &&
      owner.project()?.project.id === projectId
    );
  }
  function isCurrentObservation(epoch: number, runId: string, spaceId: string) {
    return (
      epoch === observationEpoch &&
      owner.project()?.project.activeRunId === runId &&
      store.activeSpaceId === spaceId
    );
  }

  async function generateProject() {
    const project = owner.project()?.project;
    const spaceId = store.activeSpaceId;
    if (!project || !spaceId || activeJobId.value || submitting.value) return;
    const epoch = ++submissionEpoch;
    submitting.value = true;
    try {
      const run = await generateSpeechProject({
        requestId: crypto.randomUUID(),
        projectId: project.id,
        expectedRevision: project.activeRevision,
      });
      if (!isCurrentSubmission(epoch, project.id, spaceId)) {
        observeResourceRun(run, project.title);
        return;
      }
      attachRun(run, project.id, spaceId);
      await owner.reloadProject(project.id);
      owner.setError();
    } catch (cause) {
      if (isCurrentSubmission(epoch, project.id, spaceId)) {
        owner.setError(getErrorMessage(cause));
      }
    } finally {
      if (epoch === submissionEpoch) submitting.value = false;
    }
  }

  async function cancelGeneration() {
    if (cancelling.value) return;
    cancelling.value = true;
    try {
      if (preview.jobId.value) await preview.cancel();
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

  watch(
    () => owner.project()?.project.activeRunId,
    async (runId) => {
      stopRunObservation();
      const epoch = observationEpoch;
      const projectId = owner.project()?.project.id;
      const spaceId = store.activeSpaceId;
      if (!runId || !projectId || !spaceId) return;
      try {
        const run = await invokeCommand<RunObservation>("plugin:spaces|get_run", { runId });
        if (isCurrentObservation(epoch, runId, spaceId)) attachRun(run, projectId, spaceId);
      } catch (cause) {
        if (isCurrentObservation(epoch, runId, spaceId)) {
          owner.setError(getErrorMessage(cause));
        }
      }
    },
    { immediate: true },
  );

  onScopeDispose(reset);
  return {
    preview: preview.preview,
    progress: preview.progress,
    listening: preview.listening,
    runState: readonly(runState),
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

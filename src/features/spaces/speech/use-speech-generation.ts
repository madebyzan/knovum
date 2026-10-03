import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, onScopeDispose, readonly, shallowRef } from "vue";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import { useSpaceStore } from "../session/space-store";
import type {
  SpeechAudioOutput,
  SpeechGenerationProgress,
  SpeechProjectDetail,
  SpeechProjectRenderOutput,
} from "./contracts";
import {
  cancelSpeechGeneration,
  discardSpeechPreview,
  exportSpeechRender,
  generateSpeechProject,
  previewPresetVoice,
} from "./ipc";

interface SpeechGenerationOwner {
  project: () => SpeechProjectDetail | undefined;
  reloadProject: (projectId: string) => Promise<void>;
  setError: (message?: string) => void;
}

export function useSpeechGeneration(owner: SpeechGenerationOwner) {
  const store = useSpaceStore();
  const preview = shallowRef<SpeechAudioOutput & { source: string }>();
  const renderOutput = shallowRef<SpeechProjectRenderOutput>();
  const progress = shallowRef<SpeechGenerationProgress>();
  const activeJobId = shallowRef<string>();
  const activeJobKind = shallowRef<"preview" | "project">();
  const visibleJobId = computed(() => activeJobId.value ?? owner.project()?.project.activeGenerationJobId);
  let reconnecting = false;

  function isActive(jobId: string, spaceId: string) {
    return activeJobId.value === jobId && store.activeSpaceId === spaceId;
  }

  function reportJobFailure(jobId: string, cause: unknown) {
    if (activeJobId.value === jobId && decodeAppError(cause)?.code !== "cancelled") {
      owner.setError(getErrorMessage(cause));
    }
  }

  function finishJob(jobId: string) {
    if (activeJobId.value === jobId) {
      activeJobId.value = undefined;
      activeJobKind.value = undefined;
    }
  }

  function reset() {
    const jobId = activeJobId.value;
    const previewPath = preview.value?.path;
    if (jobId && activeJobKind.value === "preview") {
      void cancelSpeechGeneration(jobId).catch((cause) => owner.setError(getErrorMessage(cause)));
    }
    if (previewPath) {
      void discardSpeechPreview(previewPath).catch((cause) => owner.setError(getErrorMessage(cause)));
    }
    activeJobId.value = undefined;
    activeJobKind.value = undefined;
    preview.value = undefined;
    renderOutput.value = undefined;
    progress.value = undefined;
  }

  async function previewSelection(text: string, voiceRevisionId: string) {
    const spaceId = store.activeSpaceId;
    if (!spaceId || activeJobId.value) return;
    const jobId = crypto.randomUUID();
    const previousPreview = preview.value?.path;
    activeJobId.value = jobId;
    activeJobKind.value = "preview";
    progress.value = undefined;
    try {
      const output = await previewPresetVoice({ jobId, spaceId, voiceRevisionId, text }, (value) => {
        if (activeJobId.value === jobId) progress.value = value;
      });
      if (!isActive(jobId, spaceId)) {
        void discardSpeechPreview(output.path).catch((cause) => owner.setError(getErrorMessage(cause)));
        return;
      }
      preview.value = { ...output, source: convertFileSrc(output.path) };
      if (previousPreview) {
        void discardSpeechPreview(previousPreview).catch((cause) => owner.setError(getErrorMessage(cause)));
      }
      owner.setError();
    } catch (cause) {
      reportJobFailure(jobId, cause);
    } finally {
      finishJob(jobId);
    }
  }

  async function generateProject() {
    const project = owner.project()?.project;
    const spaceId = store.activeSpaceId;
    if (!project || !spaceId || activeJobId.value) return;
    const jobId = crypto.randomUUID();
    activeJobId.value = jobId;
    activeJobKind.value = "project";
    renderOutput.value = undefined;
    progress.value = undefined;
    try {
      const output = await generateSpeechProject(
        { jobId, projectId: project.id, expectedRevision: project.activeRevision },
        (value) => {
          if (activeJobId.value === jobId) progress.value = value;
        },
      );
      if (!isActive(jobId, spaceId)) return;
      renderOutput.value = output;
      owner.setError();
      await owner.reloadProject(project.id);
    } catch (cause) {
      reportJobFailure(jobId, cause);
    } finally {
      finishJob(jobId);
    }
  }

  async function cancelGeneration() {
    const jobId = visibleJobId.value;
    if (!jobId) return;
    try {
      await cancelSpeechGeneration(jobId);
    } catch (cause) {
      owner.setError(getErrorMessage(cause));
    }
  }

  async function exportRender(path: string) {
    const title = owner.project()?.project.title;
    if (title) await exportSpeechRender(path, title);
  }

  const reconnectTimer = globalThis.setInterval(() => {
    const project = owner.project()?.project;
    if (!project?.activeGenerationJobId || activeJobId.value || reconnecting) return;
    reconnecting = true;
    void owner
      .reloadProject(project.id)
      .catch((cause) => owner.setError(getErrorMessage(cause)))
      .finally(() => {
        reconnecting = false;
      });
  }, 1_000);
  onScopeDispose(() => {
    globalThis.clearInterval(reconnectTimer);
    reset();
  });
  return {
    preview: readonly(preview),
    renderOutput: readonly(renderOutput),
    progress: readonly(progress),
    activeJobId: visibleJobId,
    reset,
    previewSelection,
    generateProject,
    cancelGeneration,
    exportRender,
  };
}

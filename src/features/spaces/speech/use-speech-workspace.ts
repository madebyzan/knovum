import { readonly, shallowRef, watch } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { SaveSpeechProjectRevisionRequest } from "../contracts/generated/SaveSpeechProjectRevisionRequest";
import type { SpeechProjectSnapshot } from "../contracts/generated/SpeechProjectSnapshot";
import { useSpaceStore } from "../session/space-store";
import { getSpeechProject, listSpeechProjects, saveSpeechProjectRevision } from "./ipc";
import type { NodeSpeechProjectType } from "./node-speech";
import { useSpeechGeneration } from "./use-speech-generation";
import { useSpeechProjectCreation } from "./use-speech-project-creation";
import { useSpeechVoices } from "./use-speech-voices";

export function useSpeechWorkspace(sourceNodeId: () => string, projectType: () => NodeSpeechProjectType) {
  const store = useSpaceStore();
  const projects = shallowRef<Awaited<ReturnType<typeof listSpeechProjects>>>([]);
  const loading = shallowRef(false);
  const error = shallowRef<string>();
  const selectedProject = shallowRef<SpeechProjectSnapshot>();
  const savingRevision = shallowRef(false);
  let refreshEpoch = 0;
  let selectionEpoch = 0;
  function isCurrentSpace(spaceId: string | undefined, epoch: number) {
    return store.activeSpaceId === spaceId && refreshEpoch === epoch;
  }

  function isCurrentProject(spaceId: string, epoch: number, projectId: string) {
    return isCurrentSpace(spaceId, epoch) && selectedProject.value?.project.id === projectId;
  }
  function setError(message?: string) {
    error.value = message;
  }
  function belongsToNode(project: SpeechProjectSnapshot, spaceId: string) {
    return (
      project.project.spaceId === spaceId &&
      project.project.sourceNodeId === sourceNodeId() &&
      project.project.projectType === projectType()
    );
  }

  const voiceLibrary = useSpeechVoices({
    activeSpaceId: () => store.activeSpaceId,
    contextEpoch: () => refreshEpoch,
    isCurrent: isCurrentSpace,
    setError,
  });
  const projectCreation = useSpeechProjectCreation({
    activeSpaceId: () => store.activeSpaceId,
    sourceNodeId,
    contextEpoch: () => refreshEpoch,
    isCurrent: isCurrentSpace,
    created: (summary, snapshot) => {
      projects.value = [summary, ...projects.value];
      selectedProject.value = snapshot;
    },
    setError,
  });

  const generation = useSpeechGeneration({
    project: () => selectedProject.value,
    reloadProject: async (projectId) => {
      const spaceId = store.activeSpaceId;
      const epoch = refreshEpoch;
      if (!spaceId) return;
      const project = await getSpeechProject(projectId);
      if (isCurrentProject(spaceId, epoch, projectId) && belongsToNode(project, spaceId)) {
        selectedProject.value = project;
      }
    },
    setError,
  });

  async function refresh(spaceId = store.activeSpaceId) {
    const epoch = ++refreshEpoch;
    selectionEpoch += 1;
    generation.reset();
    selectedProject.value = undefined;
    voiceLibrary.replace([]);
    projects.value = [];
    if (!spaceId) {
      loading.value = false;
      return;
    }
    loading.value = true;
    error.value = undefined;
    try {
      const [nextVoices, nextProjects] = await Promise.all([
        voiceLibrary.load(spaceId),
        listSpeechProjects(spaceId, sourceNodeId()),
      ]);
      if (epoch !== refreshEpoch) return;
      voiceLibrary.replace(nextVoices);
      projects.value = nextProjects.filter((project) => project.projectType === projectType());
    } catch (cause) {
      if (epoch === refreshEpoch) error.value = getErrorMessage(cause);
    } finally {
      if (epoch === refreshEpoch) loading.value = false;
    }
  }

  async function openProject(projectId: string) {
    const selection = ++selectionEpoch;
    const spaceId = store.activeSpaceId;
    const epoch = refreshEpoch;
    try {
      if (!spaceId) return;
      const project = await getSpeechProject(projectId);
      if (selection !== selectionEpoch || !isCurrentSpace(spaceId, epoch) || !belongsToNode(project, spaceId))
        return;
      selectedProject.value = project;
      generation.reset();
      error.value = undefined;
    } catch (cause) {
      if (selection === selectionEpoch && isCurrentSpace(spaceId, epoch)) {
        error.value = getErrorMessage(cause);
      }
    }
  }

  async function saveRevision(
    input: Omit<SaveSpeechProjectRevisionRequest, "projectId" | "expectedRevision">,
  ) {
    const current = selectedProject.value;
    if (!current || savingRevision.value) return;
    const spaceId = store.activeSpaceId;
    const epoch = refreshEpoch;
    savingRevision.value = true;
    try {
      if (!spaceId || current.project.spaceId !== spaceId) return;
      const snapshot = await saveSpeechProjectRevision({
        ...input,
        projectId: current.project.id,
        expectedRevision: current.project.activeRevision,
      });
      if (!isCurrentProject(spaceId, epoch, current.project.id) || snapshot.project.spaceId !== spaceId)
        return;
      generation.clearPreview();
      selectedProject.value = snapshot;
      const updated = snapshot.project;
      projects.value = projects.value.map((project) => (project.id === updated.id ? updated : project));
      error.value = undefined;
    } catch (cause) {
      if (spaceId && isCurrentProject(spaceId, epoch, current.project.id))
        error.value = getErrorMessage(cause);
    } finally {
      savingRevision.value = false;
    }
  }

  watch([() => store.activeSpaceId, sourceNodeId, projectType], () => void refresh(), {
    immediate: true,
    flush: "sync",
  });
  return {
    voices: voiceLibrary.voices,
    creatingVoice: voiceLibrary.creating,
    addPresetVoice: voiceLibrary.addPreset,
    projects: readonly(projects),
    loading: readonly(loading),
    error: readonly(error),
    selectedProject: readonly(selectedProject),
    creatingProject: projectCreation.creating,
    savingRevision: readonly(savingRevision),
    preview: generation.preview,
    previewListening: generation.listening,
    runState: generation.runState,
    generationProgress: generation.progress,
    activeJobId: generation.activeJobId,
    generationSubmitting: generation.submitting,
    generationCancelling: generation.cancelling,
    addProject: projectCreation.add,
    refreshVoices: voiceLibrary.refresh,
    openProject,
    saveRevision,
    previewSelection: generation.previewSelection,
    generateProject: generation.generateProject,
    cancelGeneration: generation.cancelGeneration,
    stopPreviewListening: generation.stopListening,
    exportRender: generation.exportRender,
  };
}

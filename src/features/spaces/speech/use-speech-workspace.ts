import { readonly, shallowRef, watch } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { useSpaceStore } from "../session/space-store";
import type { SaveSpeechProjectRevisionInput, SpeechProjectDetail } from "./contracts";
import { getSpeechProject, listSpeechProjects, saveSpeechProjectRevision } from "./ipc";
import { useSpeechGeneration } from "./use-speech-generation";
import { useSpeechProjectCreation } from "./use-speech-project-creation";
import { useSpeechVoices } from "./use-speech-voices";

export function useSpeechWorkspace() {
  const store = useSpaceStore();
  const projects = shallowRef<Awaited<ReturnType<typeof listSpeechProjects>>>([]);
  const loading = shallowRef(false);
  const error = shallowRef<string>();
  const selectedProject = shallowRef<SpeechProjectDetail>();
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

  const voiceLibrary = useSpeechVoices({
    activeSpaceId: () => store.activeSpaceId,
    contextEpoch: () => refreshEpoch,
    isCurrent: isCurrentSpace,
    setError,
  });
  const projectCreation = useSpeechProjectCreation({
    activeSpaceId: () => store.activeSpaceId,
    contextEpoch: () => refreshEpoch,
    isCurrent: isCurrentSpace,
    created: (summary, detail) => {
      projects.value = [summary, ...projects.value];
      selectedProject.value = detail;
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
      if (isCurrentProject(spaceId, epoch, projectId) && project.project.spaceId === spaceId) {
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
        listSpeechProjects(spaceId),
      ]);
      if (epoch !== refreshEpoch) return;
      voiceLibrary.replace(nextVoices);
      projects.value = nextProjects;
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
      if (
        selection !== selectionEpoch ||
        !isCurrentSpace(spaceId, epoch) ||
        project.project.spaceId !== spaceId
      )
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

  async function saveRevision(input: Omit<SaveSpeechProjectRevisionInput, "projectId" | "expectedRevision">) {
    const current = selectedProject.value;
    if (!current || savingRevision.value) return;
    const spaceId = store.activeSpaceId;
    const epoch = refreshEpoch;
    savingRevision.value = true;
    try {
      if (!spaceId || current.project.spaceId !== spaceId) return;
      const detail = await saveSpeechProjectRevision({
        ...input,
        projectId: current.project.id,
        expectedRevision: current.project.activeRevision,
      });
      if (!isCurrentProject(spaceId, epoch, current.project.id) || detail.project.spaceId !== spaceId) return;
      generation.clearPreview();
      selectedProject.value = detail;
      const updated = detail.project;
      projects.value = projects.value.map((project) => (project.id === updated.id ? updated : project));
      error.value = undefined;
    } catch (cause) {
      if (spaceId && isCurrentProject(spaceId, epoch, current.project.id))
        error.value = getErrorMessage(cause);
    } finally {
      savingRevision.value = false;
    }
  }

  watch(
    () => store.activeSpaceId,
    (spaceId) => void refresh(spaceId),
    { immediate: true },
  );
  return {
    voices: voiceLibrary.voices,
    projects: readonly(projects),
    loading: readonly(loading),
    error: readonly(error),
    selectedProject: readonly(selectedProject),
    creatingVoice: voiceLibrary.creating,
    creatingProject: projectCreation.creating,
    savingRevision: readonly(savingRevision),
    preview: generation.preview,
    previewListening: generation.listening,
    runState: generation.runState,
    generationProgress: generation.progress,
    activeJobId: generation.activeJobId,
    generationSubmitting: generation.submitting,
    generationCancelling: generation.cancelling,
    addPreset: voiceLibrary.addPreset,
    addProject: projectCreation.add,
    acceptVoice: voiceLibrary.accept,
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

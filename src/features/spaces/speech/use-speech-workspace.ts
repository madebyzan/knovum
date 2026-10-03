import { readonly, shallowRef, watch } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { useSpaceStore } from "../session/space-store";
import type {
  CreatePresetVoiceInput,
  CreateSpeechProjectInput,
  SaveSpeechProjectRevisionInput,
  SpeechProjectDetail,
} from "./contracts";
import {
  createPresetVoice,
  createSpeechProject,
  getSpeechProject,
  listSpeechProjects,
  listVoiceProfiles,
  saveSpeechProjectRevision,
} from "./ipc";
import { useSpeechGeneration } from "./use-speech-generation";

export function useSpeechWorkspace() {
  const store = useSpaceStore();
  const voices = shallowRef<Awaited<ReturnType<typeof listVoiceProfiles>>>([]);
  const projects = shallowRef<Awaited<ReturnType<typeof listSpeechProjects>>>([]);
  const loading = shallowRef(false);
  const error = shallowRef<string>();
  const selectedProject = shallowRef<SpeechProjectDetail>();
  let refreshEpoch = 0;
  function isCurrentSpace(spaceId: string | undefined, epoch: number) {
    return store.activeSpaceId === spaceId && refreshEpoch === epoch;
  }

  function isCurrentProject(spaceId: string, epoch: number, projectId: string) {
    return isCurrentSpace(spaceId, epoch) && selectedProject.value?.project.id === projectId;
  }

  const generation = useSpeechGeneration({
    project: () => selectedProject.value,
    reloadProject: async (projectId) => {
      const spaceId = store.activeSpaceId;
      const epoch = refreshEpoch;
      if (!spaceId) return;
      const project = await getSpeechProject(projectId);
      if (isCurrentSpace(spaceId, epoch) && project.project.spaceId === spaceId) {
        selectedProject.value = project;
      }
    },
    setError: (message) => {
      error.value = message;
    },
  });

  async function refresh(spaceId = store.activeSpaceId) {
    const epoch = ++refreshEpoch;
    generation.reset();
    selectedProject.value = undefined;
    if (!spaceId) {
      voices.value = [];
      projects.value = [];
      selectedProject.value = undefined;
      loading.value = false;
      return;
    }
    loading.value = true;
    error.value = undefined;
    try {
      const [nextVoices, nextProjects] = await Promise.all([
        listVoiceProfiles(spaceId),
        listSpeechProjects(spaceId),
      ]);
      if (epoch !== refreshEpoch) return;
      voices.value = nextVoices;
      projects.value = nextProjects;
    } catch (cause) {
      if (epoch === refreshEpoch) error.value = getErrorMessage(cause);
    } finally {
      if (epoch === refreshEpoch) loading.value = false;
    }
  }

  async function addPreset(input: Omit<CreatePresetVoiceInput, "spaceId">) {
    const spaceId = store.activeSpaceId;
    const epoch = refreshEpoch;
    try {
      if (!spaceId) throw new Error("A persisted Space is required");
      const voice = await createPresetVoice({ ...input, spaceId });
      if (!isCurrentSpace(spaceId, epoch)) return;
      voices.value = [voice, ...voices.value];
      error.value = undefined;
    } catch (cause) {
      if (isCurrentSpace(spaceId, epoch)) error.value = getErrorMessage(cause);
    }
  }

  async function addProject(input: Omit<CreateSpeechProjectInput, "spaceId">) {
    if (generation.activeJobId.value) return;
    const spaceId = store.activeSpaceId;
    const epoch = refreshEpoch;
    try {
      if (!spaceId) throw new Error("A persisted Space is required");
      const project = await createSpeechProject({ ...input, spaceId });
      if (!isCurrentSpace(spaceId, epoch)) return;
      projects.value = [project, ...projects.value];
      const detail = await getSpeechProject(project.id);
      if (!isCurrentSpace(spaceId, epoch) || detail.project.spaceId !== spaceId) return;
      selectedProject.value = detail;
      error.value = undefined;
    } catch (cause) {
      if (isCurrentSpace(spaceId, epoch)) error.value = getErrorMessage(cause);
    }
  }

  async function openProject(projectId: string) {
    if (generation.activeJobId.value) return;
    const spaceId = store.activeSpaceId;
    const epoch = refreshEpoch;
    try {
      if (!spaceId) return;
      const project = await getSpeechProject(projectId);
      if (!isCurrentSpace(spaceId, epoch) || project.project.spaceId !== spaceId) return;
      selectedProject.value = project;
      generation.reset();
      error.value = undefined;
    } catch (cause) {
      if (isCurrentSpace(spaceId, epoch)) error.value = getErrorMessage(cause);
    }
  }

  async function saveRevision(input: Omit<SaveSpeechProjectRevisionInput, "projectId" | "expectedRevision">) {
    const current = selectedProject.value;
    if (!current || current.project.projectType !== "short_dubbing") return;
    const spaceId = store.activeSpaceId;
    const epoch = refreshEpoch;
    try {
      if (!spaceId || current.project.spaceId !== spaceId) return;
      const detail = await saveSpeechProjectRevision({
        ...input,
        projectId: current.project.id,
        expectedRevision: current.project.activeRevision,
      });
      if (!isCurrentProject(spaceId, epoch, current.project.id) || detail.project.spaceId !== spaceId) return;
      selectedProject.value = detail;
      const updated = detail.project;
      projects.value = projects.value.map((project) => (project.id === updated.id ? updated : project));
      error.value = undefined;
    } catch (cause) {
      if (spaceId && isCurrentProject(spaceId, epoch, current.project.id))
        error.value = getErrorMessage(cause);
    }
  }

  watch(
    () => store.activeSpaceId,
    (spaceId) => void refresh(spaceId),
    { immediate: true },
  );
  return {
    voices: readonly(voices),
    projects: readonly(projects),
    loading: readonly(loading),
    error: readonly(error),
    selectedProject: readonly(selectedProject),
    preview: generation.preview,
    renderOutput: generation.renderOutput,
    generationProgress: generation.progress,
    activeJobId: generation.activeJobId,
    addPreset,
    addProject,
    openProject,
    saveRevision,
    previewSelection: generation.previewSelection,
    generateProject: generation.generateProject,
    cancelGeneration: generation.cancelGeneration,
    exportRender: generation.exportRender,
  };
}

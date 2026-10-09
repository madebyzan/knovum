import { computed, readonly, shallowRef, watch } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { SaveSpeechProjectRevisionRequest } from "../contracts/generated/SaveSpeechProjectRevisionRequest";
import type { SpeechProjectSnapshot } from "../contracts/generated/SpeechProjectSnapshot";
import { useSpaceStore } from "../session/space-store";
import { getSpeechProject, listSpeechProjects, saveSpeechProjectRevision } from "./ipc";
import type { NodeSpeechProjectType } from "./node-speech";
import { useSpeechGeneration } from "./use-speech-generation";
import { useSpeechProjectCreation } from "./use-speech-project-creation";
import { useSpeechVoices } from "./use-speech-voices";

type SpeechRevisionInput = Omit<SaveSpeechProjectRevisionRequest, "projectId" | "expectedRevision">;

export function useSpeechWorkspace(sourceNodeId: () => string, projectType: () => NodeSpeechProjectType) {
  const store = useSpaceStore();
  const projects = shallowRef<Awaited<ReturnType<typeof listSpeechProjects>>>([]);
  const loading = shallowRef(false);
  const workspaceError = shallowRef<string>();
  const selectedProject = shallowRef<SpeechProjectSnapshot>();
  const revisionSave = shallowRef<Promise<boolean>>();
  const savingRevision = computed(() => Boolean(revisionSave.value));
  let refreshEpoch = 0;
  let selectionEpoch = 0;
  function isCurrentSpace(spaceId: string | undefined, epoch: number) {
    return store.activeSpaceId === spaceId && refreshEpoch === epoch;
  }
  function isCurrentProject(spaceId: string, epoch: number, projectId: string) {
    return isCurrentSpace(spaceId, epoch) && selectedProject.value?.project.id === projectId;
  }
  function belongsToNode(project: SpeechProjectSnapshot, spaceId: string) {
    return (
      project.project.spaceId === spaceId &&
      project.project.sourceNodeId === sourceNodeId() &&
      project.project.projectType === projectType()
    );
  }
  const voiceLibrary = useSpeechVoices();
  const projectCreation = useSpeechProjectCreation({
    activeSpaceId: () => store.activeSpaceId,
    sourceNodeId,
    contextEpoch: () => refreshEpoch,
    isCurrent: isCurrentSpace,
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
  });
  const errors = computed(() => [
    ...new Set(
      [
        workspaceError.value,
        projectCreation.error.value,
        ...generation.errors.value,
        ...voiceLibrary.errors.value,
      ].filter((message): message is string => Boolean(message)),
    ),
  ]);

  async function refresh(spaceId = store.activeSpaceId) {
    const epoch = ++refreshEpoch;
    selectionEpoch += 1;
    generation.reset();
    projectCreation.reset();
    selectedProject.value = undefined;
    projects.value = [];
    workspaceError.value = undefined;
    if (!spaceId) {
      loading.value = false;
      return;
    }
    loading.value = true;
    try {
      const nextProjects = await listSpeechProjects(spaceId, sourceNodeId());
      if (epoch !== refreshEpoch) return;
      projects.value = nextProjects.filter((project) => project.projectType === projectType());
    } catch (cause) {
      if (epoch === refreshEpoch) workspaceError.value = getErrorMessage(cause);
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
      workspaceError.value = undefined;
    } catch (cause) {
      if (selection === selectionEpoch && isCurrentSpace(spaceId, epoch)) {
        workspaceError.value = getErrorMessage(cause);
      }
    }
  }

  async function addProject(input: Parameters<typeof projectCreation.add>[0]) {
    const created = await projectCreation.add(input);
    if (!created) return;
    projects.value = [created.summary, ...projects.value];
    selectedProject.value = created.snapshot;
  }

  function saveRevision(input: SpeechRevisionInput) {
    revisionSave.value ??= persistRevision(input).finally(() => {
      revisionSave.value = undefined;
    });
    return revisionSave.value;
  }

  async function settleRevisionSave() {
    await revisionSave.value;
  }

  async function persistRevision(input: SpeechRevisionInput) {
    const current = selectedProject.value;
    if (!current) return false;
    const spaceId = store.activeSpaceId;
    const epoch = refreshEpoch;
    try {
      if (!spaceId || current.project.spaceId !== spaceId) return false;
      const snapshot = await saveSpeechProjectRevision({
        ...input,
        projectId: current.project.id,
        expectedRevision: current.project.activeRevision,
      });
      if (!isCurrentProject(spaceId, epoch, current.project.id) || snapshot.project.spaceId !== spaceId)
        return false;
      generation.clearPreview();
      selectedProject.value = snapshot;
      const updated = snapshot.project;
      projects.value = projects.value.map((project) => (project.id === updated.id ? updated : project));
      workspaceError.value = undefined;
      return true;
    } catch (cause) {
      if (spaceId && isCurrentProject(spaceId, epoch, current.project.id))
        workspaceError.value = getErrorMessage(cause);
      return false;
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
    loading: computed(() => loading.value || voiceLibrary.loading.value),
    errors,
    selectedProject: readonly(selectedProject),
    creatingProject: projectCreation.creating,
    savingRevision,
    preview: generation.preview,
    previewListening: generation.listening,
    runState: generation.runState,
    generationProgress: generation.progress,
    activeJobId: generation.activeJobId,
    generationSubmitting: generation.submitting,
    generationCancelling: generation.cancelling,
    addProject,
    openProject,
    saveRevision,
    settleRevisionSave,
    previewSelection: generation.previewSelection,
    generateProject: generation.generateProject,
    cancelGeneration: generation.cancelGeneration,
    stopPreviewListening: generation.stopListening,
    exportRender: generation.exportRender,
  };
}

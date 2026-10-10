import { computed, onScopeDispose, readonly, shallowRef, watch } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { SpeechProjectSnapshot } from "../contracts/generated/SpeechProjectSnapshot";
import { useSpaceStore } from "../session/space-store";
import { getSpeechProject, listSpeechProjects } from "./ipc";
import type { NodeSpeechProjectType } from "./node-speech";
import { useSpeechGeneration } from "./use-speech-generation";
import { useSpeechProjectCreation } from "./use-speech-project-creation";
import { useSpeechProjectRevision, type SpeechRevisionInput } from "./use-speech-project-revision";
import { useSpeechVoices } from "./use-speech-voices";

export function useSpeechWorkspace(sourceNodeId: () => string, projectType: () => NodeSpeechProjectType) {
  const store = useSpaceStore();
  const projects = shallowRef<Awaited<ReturnType<typeof listSpeechProjects>>>([]);
  const loading = shallowRef(false);
  const workspaceError = shallowRef<string>();
  const {
    selectedProject,
    saving: savingRevision,
    error: revisionError,
    select: selectRevision,
    save: saveRevision,
    settle: settleRevisionSave,
  } = useSpeechProjectRevision();
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
        selectRevision(project);
      }
    },
  });
  watch(
    selectedProject,
    (snapshot, previous) => {
      if (!snapshot) return;
      projects.value = projects.value.map((project) =>
        project.id === snapshot.project.id ? snapshot.project : project,
      );
      if (previous && snapshot.revision.id !== previous.revision.id) generation.clearPreview();
    },
    { flush: "sync" },
  );
  const errors = computed(() => [
    ...new Set(
      [
        workspaceError.value,
        projectCreation.error.value,
        revisionError.value,
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
    selectRevision();
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
      const latest = projects.value.toSorted((a, b) => b.updatedAt - a.updatedAt)[0];
      if (latest) await openProject(latest.id);
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
      selectRevision(project);
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
    selectRevision(created.snapshot);
  }

  async function startGeneration(input?: SpeechRevisionInput) {
    if (savingRevision.value || generation.hasActiveExecution.value || generation.submitting.value) return;
    const projectId = selectedProject.value?.project.id;
    const spaceId = store.activeSpaceId;
    const epoch = refreshEpoch;
    if (!projectId || !spaceId) return;
    if (input && !(await saveRevision(input))) return;
    if (isCurrentProject(spaceId, epoch, projectId)) await generation.generateProject();
  }

  watch([() => store.activeSpaceId, sourceNodeId, projectType], () => void refresh(), {
    immediate: true,
    flush: "sync",
  });
  onScopeDispose(() => {
    refreshEpoch += 1;
  });
  return {
    voices: voiceLibrary.voices,
    creatingVoice: voiceLibrary.creating,
    addPresetVoice: voiceLibrary.addPreset,
    projects: readonly(projects),
    initializationReady: computed(() => !loading.value && !workspaceError.value && !projects.value.length),
    loading: computed(() => loading.value || voiceLibrary.loading.value),
    errors,
    selectedProject,
    creatingProject: projectCreation.creating,
    savingRevision,
    preview: generation.preview,
    previewListening: generation.listening,
    runState: generation.runState,
    generationProgress: generation.progress,
    hasActiveExecution: generation.hasActiveExecution,
    generationSubmitting: generation.submitting,
    generationCancelling: generation.cancelling,
    addProject,
    refresh,
    openProject,
    saveRevision,
    settleRevisionSave,
    previewSelection: generation.previewSelection,
    startGeneration,
    cancelGeneration: generation.cancelGeneration,
    stopPreviewListening: generation.stopListening,
    exportRender: generation.exportRender,
  };
}

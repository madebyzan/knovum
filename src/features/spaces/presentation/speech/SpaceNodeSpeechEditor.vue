<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";

import { isApplicationLeaveCancelled } from "../../../../shared/contracts/application-leave";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import UiSpinner from "../../../../shared/ui/UiSpinner.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { NodeSpeechProjectType } from "../../speech/node-speech";
import { publishedSpeechOutput } from "../../speech/published-speech-output";
import { speechProjectInput } from "../../speech/speech-project-input";
import { useSpeechMediaSources } from "../../speech/use-speech-media-sources";
import { useSpeechProjectLeave } from "../../speech/use-speech-project-leave";
import { useSpeechWorkspace } from "../../speech/use-speech-workspace";
import SpaceUnsavedChangesDialog from "../workspace/SpaceUnsavedChangesDialog.vue";
import SpeechNodeEditorHeader from "./SpeechNodeEditorHeader.vue";
import SpeechProjectEditor from "./SpeechProjectEditor.vue";
import SpeechProjectSelection from "./SpeechProjectSelection.vue";
import SpeechVoiceAccess from "./SpeechVoiceAccess.vue";
import VoiceCandidateForm from "./VoiceCandidateForm.vue";

const emit = defineEmits<{ close: [] }>();
const props = defineProps<{ node: SpaceCanvasNode; projectType: NodeSpeechProjectType }>();
const {
  voices,
  projects,
  initializationReady,
  selectedProject,
  preview,
  previewListening,
  runState,
  generationProgress,
  hasActiveExecution,
  creatingProject,
  savingRevision,
  generationSubmitting,
  generationCancelling,
  loading,
  errors,
  addProject,
  refresh,
  openProject,
  saveRevision,
  settleRevisionSave,
  previewSelection,
  startGeneration,
  cancelGeneration,
  stopPreviewListening,
  exportRender,
} = useSpeechWorkspace(
  () => props.node.id,
  () => props.projectType,
);

const isAddingVoice = shallowRef(false);
const { videos } = useSpeechMediaSources();
const initialInput = computed(() =>
  speechProjectInput(
    props.node,
    props.projectType,
    videos.value.find((video) => video.id === props.node.id)?.source,
  ),
);
const initializationAttempted = shallowRef(false);
watch(
  [loading, initialInput, initializationReady],
  () => {
    if (loading.value || !initializationReady.value || selectedProject.value) return;
    if (!initialInput.value || initializationAttempted.value) return;
    initializationAttempted.value = true;
    void addProject(initialInput.value);
  },
  { immediate: true },
);
const draftEpoch = shallowRef(0);
const projectLeave = useSpeechProjectLeave(
  saveRevision,
  openProject,
  () => (draftEpoch.value += 1),
  settleRevisionSave,
);

async function requestClose() {
  try {
    await projectLeave.leave.requestLeave();
    return true;
  } catch (error) {
    if (isApplicationLeaveCancelled(error)) return false;
    throw error;
  }
}
async function createProject() {
  if (!initialInput.value || creatingProject.value || !(await requestClose())) return;
  await addProject(initialInput.value);
}
async function retryInitialization() {
  initializationAttempted.value = false;
  await refresh();
}
defineExpose({ requestClose });
</script>

<template>
  <div class="flex min-h-0 flex-col" :aria-busy="loading || creatingProject">
    <SpeechNodeEditorHeader
      :node
      :project-type
      :is-adding-voice="isAddingVoice"
      @back="isAddingVoice = false"
      @close="emit('close')"
    />
    <div v-if="errors.length" class="space-y-1 bg-surface-danger-subtle px-3 py-2" role="alert">
      <p v-for="message in errors" :key="message" class="text-caption text-fg-danger">{{ message }}</p>
    </div>
    <div v-if="loading || creatingProject" class="flex items-center gap-2 p-3 text-caption text-fg-secondary">
      <UiSpinner size="sm" />{{ $t("common.loading") }}
    </div>
    <SpeechVoiceAccess
      v-if="!loading && !creatingProject && !isAddingVoice"
      :empty="!voices.some((voice) => voice.isActive && voice.archivedAt == null)"
      :disabled="hasActiveExecution || savingRevision || generationSubmitting"
      @add="isAddingVoice = true"
    >
      <UiButton
        v-if="selectedProject"
        size="sm"
        variant="ghost"
        class="ml-auto"
        :disabled="hasActiveExecution || savingRevision || generationSubmitting"
        @click="createProject"
        >{{ $t("spaces.speech.project.new") }}</UiButton
      >
    </SpeechVoiceAccess>
    <div v-if="!loading && !creatingProject && !selectedProject" class="space-y-2 p-3">
      <p v-if="!initialInput" class="text-caption text-fg-secondary" role="status">
        {{ $t("spaces.speech.project.sourceNotReady") }}
      </p>
      <UiButton v-else size="sm" @click="retryInitialization">{{ $t("common.retry") }}</UiButton>
    </div>
    <UiScrollArea v-if="isAddingVoice" class="max-h-96 min-h-0" viewport-class="max-h-[inherit]">
      <div class="p-3"><VoiceCandidateForm /></div>
    </UiScrollArea>
    <SpeechProjectSelection
      v-if="selectedProject && projects.length > 1 && !isAddingVoice"
      :projects="projects"
      :selected-id="selectedProject.project.id"
      :disabled="creatingProject || savingRevision || hasActiveExecution || generationSubmitting"
      @select="projectLeave.selectProject"
    />
    <SpeechProjectEditor
      v-if="selectedProject"
      :visible="!isAddingVoice"
      :key="`${selectedProject.project.id}:${draftEpoch}`"
      :source-node="node"
      :project="selectedProject"
      :voices="voices"
      :preview="preview"
      :preview-listening="previewListening"
      :output="publishedSpeechOutput(selectedProject.revision)"
      :progress="generationProgress"
      :run-state="runState"
      :running="hasActiveExecution"
      :saving="savingRevision"
      :submitting="generationSubmitting"
      :cancelling="generationCancelling"
      @draft-state="projectLeave.updateDraft"
      @preview="previewSelection"
      @generate="startGeneration"
      @cancel="cancelGeneration"
      @stop-listening="stopPreviewListening"
      @export="exportRender"
    />
  </div>
  <SpaceUnsavedChangesDialog
    :open="projectLeave.leave.dialogOpen.value"
    :saving="projectLeave.leave.saving.value"
    :show-save="Boolean(projectLeave.draft.value)"
    :title="projectLeave.dialogTitle.value"
    :description="projectLeave.dialogDescription.value"
    @cancel="projectLeave.leave.cancel"
    @discard="projectLeave.leave.discard"
    @save="projectLeave.leave.save"
  />
</template>

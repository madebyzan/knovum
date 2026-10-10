<script setup lang="ts">
import { computed, shallowRef } from "vue";

import { isApplicationLeaveCancelled } from "../../../../shared/contracts/application-leave";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiSpinner from "../../../../shared/ui/UiSpinner.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { NodeSpeechProjectType } from "../../speech/node-speech";
import { publishedSpeechOutput } from "../../speech/published-speech-output";
import { useSpeechProjectLeave } from "../../speech/use-speech-project-leave";
import { useSpeechWorkspace } from "../../speech/use-speech-workspace";
import SpaceUnsavedChangesDialog from "../workspace/SpaceUnsavedChangesDialog.vue";
import SpeechNodeEditorHeader from "./SpeechNodeEditorHeader.vue";
import SpeechProjectEditor from "./SpeechProjectEditor.vue";
import SpeechProjectForm from "./SpeechProjectForm.vue";
import VoiceCandidateForm from "./VoiceCandidateForm.vue";

const emit = defineEmits<{ close: [] }>();
const props = defineProps<{ node: SpaceCanvasNode; projectType: NodeSpeechProjectType }>();
const {
  voices,
  projects,
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
  openProject,
  saveRevision,
  settleRevisionSave,
  previewSelection,
  generateProject,
  cancelGeneration,
  stopPreviewListening,
  exportRender,
} = useSpeechWorkspace(
  () => props.node.id,
  () => props.projectType,
);

const isAddingVoice = shallowRef(false);
const isAddingProject = shallowRef(false);
const draftEpoch = shallowRef(0);
const projectLeave = useSpeechProjectLeave(
  saveRevision,
  openProject,
  () => (draftEpoch.value += 1),
  settleRevisionSave,
);

const projectOptions = computed(() =>
  projects.value.map((project) => ({ value: project.id, label: project.title })),
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
async function toggleNewProject() {
  if (!isAddingProject.value && !(await requestClose())) return;
  isAddingProject.value = !isAddingProject.value;
}
async function createProject(input: Parameters<typeof addProject>[0]) {
  const previous = selectedProject.value?.project.id;
  await addProject(input);
  if (selectedProject.value?.project.id !== previous) isAddingProject.value = false;
}
defineExpose({ requestClose });
</script>

<template>
  <div class="flex min-h-0 flex-col" :aria-busy="loading">
    <SpeechNodeEditorHeader
      :node
      :project-type
      :revision="selectedProject?.project.activeRevision"
      :is-creating="creatingProject"
      :is-adding-project="isAddingProject"
      :is-adding-voice="isAddingVoice"
      @new-project="toggleNewProject"
      @add-voice="isAddingVoice = !isAddingVoice"
      @close="emit('close')"
    />
    <div v-if="errors.length" class="space-y-1 bg-surface-danger-subtle px-3 py-2" role="alert">
      <p v-for="message in errors" :key="message" class="text-caption text-fg-danger">{{ message }}</p>
    </div>
    <div v-if="loading" class="flex items-center gap-2 p-3 text-caption text-fg-secondary">
      <UiSpinner size="sm" />{{ $t("common.loading") }}
    </div>
    <div v-if="projects.length > 1" class="shrink-0 px-3 py-2">
      <UiSelect
        size="sm"
        :model-value="selectedProject?.project.id ?? ''"
        :options="projectOptions"
        :label="$t('spaces.speech.project.nodeProjects')"
        @update:model-value="projectLeave.selectProject"
      />
    </div>
    <p
      v-else-if="selectedProject && selectedProject.project.title !== node.data.title"
      class="truncate px-3 py-2 text-caption text-fg-secondary"
      :title="selectedProject.project.title"
    >
      {{ selectedProject.project.title }}
    </p>
    <UiScrollArea
      v-if="isAddingVoice || (!loading && (!selectedProject || isAddingProject))"
      class="max-h-96 min-h-0"
      viewport-class="max-h-[inherit]"
    >
      <div class="p-3">
        <VoiceCandidateForm v-if="isAddingVoice" />
        <SpeechProjectForm
          v-if="!selectedProject || isAddingProject"
          v-show="!isAddingVoice"
          :node="node"
          :project-type="projectType"
          :disabled="creatingProject"
          @create="createProject"
        />
      </div>
    </UiScrollArea>
    <SpeechProjectEditor
      v-if="selectedProject"
      :visible="!isAddingProject && !isAddingVoice"
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
      @save="saveRevision"
      @draft-state="projectLeave.updateDraft"
      @preview="previewSelection"
      @generate="generateProject"
      @cancel="cancelGeneration"
      @stop-listening="stopPreviewListening"
      @export="exportRender"
    />
  </div>
  <SpaceUnsavedChangesDialog
    :open="projectLeave.leave.dialogOpen.value"
    :saving="projectLeave.leave.saving.value"
    :can-save="Boolean(projectLeave.draft.value)"
    :title="$t('spaces.speech.project.unsavedTitle')"
    :description="$t('spaces.speech.project.unsavedDescription')"
    @cancel="projectLeave.leave.cancel"
    @discard="projectLeave.leave.discard"
    @save="projectLeave.leave.save"
  />
</template>

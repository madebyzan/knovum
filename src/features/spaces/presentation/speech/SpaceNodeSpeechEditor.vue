<script setup lang="ts">
import { Plus, Settings2 } from "@lucide/vue";
import { computed, shallowRef } from "vue";
import { RouterLink } from "vue-router";

import { isApplicationLeaveCancelled } from "../../../../shared/contracts/application-leave";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { NodeSpeechProjectType } from "../../speech/node-speech";
import { publishedSpeechOutput } from "../../speech/published-speech-output";
import { useSpeechProjectLeave } from "../../speech/use-speech-project-leave";
import { useSpeechWorkspace } from "../../speech/use-speech-workspace";
import SpaceUnsavedChangesDialog from "../workspace/SpaceUnsavedChangesDialog.vue";
import SpeechProjectEditor from "./SpeechProjectEditor.vue";
import SpeechProjectForm from "./SpeechProjectForm.vue";
import VoiceCandidateForm from "./VoiceCandidateForm.vue";

const props = defineProps<{ node: SpaceCanvasNode; projectType: NodeSpeechProjectType }>();
const {
  voices,
  projects,
  selectedProject,
  preview,
  previewListening,
  runState,
  generationProgress,
  activeJobId,
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

const addingVoice = shallowRef(false);
const addingProject = shallowRef(false);
const draftEpoch = shallowRef(0);
const projectLeave = useSpeechProjectLeave(
  saveRevision,
  openProject,
  () => (draftEpoch.value += 1),
  settleRevisionSave,
);

const publishedOutput = computed(() => publishedSpeechOutput(selectedProject.value?.revision));
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
  if (!addingProject.value && !(await requestClose())) return;
  addingProject.value = !addingProject.value;
}
async function createProject(input: Parameters<typeof addProject>[0]) {
  const previous = selectedProject.value?.project.id;
  await addProject(input);
  if (selectedProject.value?.project.id !== previous) addingProject.value = false;
}
defineExpose({ requestClose });
</script>

<template>
  <div>
    <div v-if="errors.length" class="mb-3 space-y-1 rounded-md bg-surface-danger-subtle p-2">
      <p v-for="message in errors" :key="message" class="text-caption text-fg-danger">{{ message }}</p>
    </div>
    <div class="space-y-3" :aria-busy="loading">
      <div class="flex items-center justify-between gap-2">
        <RouterLink
          :to="{ path: '/settings', query: { pane: 'voices' } }"
          class="inline-flex items-center gap-1.5 text-caption text-fg-secondary hover:text-fg-primary"
          data-testid="manage-voices"
        >
          <Settings2 class="size-3.5" />{{ $t("spaces.speech.library.manage") }}
        </RouterLink>
        <UiButton
          size="sm"
          variant="ghost"
          data-testid="add-reference-voice"
          @click="addingVoice = !addingVoice"
        >
          {{ $t(addingVoice ? "common.back" : "spaces.speech.library.add") }}
        </UiButton>
      </div>
      <VoiceCandidateForm v-if="addingVoice" />
      <div v-show="!addingVoice" class="space-y-3">
        <section v-if="!selectedProject || addingProject" class="space-y-3">
          <h2 class="text-label font-medium text-fg-primary">{{ $t("spaces.speech.project.new") }}</h2>
          <fieldset>
            <SpeechProjectForm
              :node="node"
              :project-type="projectType"
              :disabled="creatingProject"
              @create="createProject"
            />
          </fieldset>
        </section>
        <div v-if="projects.length" class="flex items-center gap-2">
          <UiSelect
            class="min-w-0 flex-1"
            :model-value="selectedProject?.project.id ?? ''"
            :options="projectOptions"
            :label="$t('spaces.speech.project.nodeProjects')"
            @update:model-value="projectLeave.selectProject"
          />
          <UiButton size="sm" variant="ghost" :disabled="creatingProject" @click="toggleNewProject">
            <Plus class="size-3.5" />{{ $t(addingProject ? "common.cancel" : "spaces.speech.project.new") }}
          </UiButton>
        </div>
        <SpeechProjectEditor
          v-if="selectedProject"
          v-show="!addingProject"
          :key="`${selectedProject.project.id}:${draftEpoch}`"
          :source-node="node"
          :project="selectedProject"
          :voices="voices"
          :preview="preview"
          :preview-listening="previewListening"
          :output="publishedOutput"
          :progress="generationProgress"
          :run-state="runState"
          :running="Boolean(activeJobId)"
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
    </div>
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

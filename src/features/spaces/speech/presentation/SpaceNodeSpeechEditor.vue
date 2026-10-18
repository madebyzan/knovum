<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";

import { isApplicationLeaveCancelled } from "../../../../shared/contracts/application-leave";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSpinner from "../../../../shared/ui/UiSpinner.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import SpaceUnsavedChangesDialog from "../../workspace/presentation/SpaceUnsavedChangesDialog.vue";
import type { NodeSpeechProjectType } from "../node-speech";
import { publishedSpeechOutput } from "../published-speech-output";
import { speechProjectInput } from "../speech-project-input";
import { useSpeechMediaSources } from "../use-speech-media-sources";
import { useSpeechProjectLeave } from "../use-speech-project-leave";
import { useSpeechWorkspace } from "../use-speech-workspace";
import SpeechNodeEditorHeader from "./SpeechNodeEditorHeader.vue";
import SpeechProjectEditor from "./SpeechProjectEditor.vue";

const emit = defineEmits<{ close: [] }>();
const props = defineProps<{ node: SpaceCanvasNode; projectType: NodeSpeechProjectType }>();
const {
  voices,
  initializationReady,
  selectedProject,
  preview,
  previewListening,
  previewProgress,
  previewing,
  generating,
  creatingProject,
  savingRevision,
  generationSubmitting,
  previewCancelling,
  loading,
  errors,
  addProject,
  refresh,
  openProject,
  saveRevision,
  settleRevisionSave,
  previewSelection,
  startGeneration,
  cancelPreview,
  stopPreviewListening,
} = useSpeechWorkspace(
  () => props.node.id,
  () => props.projectType,
);

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
/**
 * Generating hands the work to the Task Center, which reports and cancels it, and to the canvas,
 * where the output node is already waiting for it. The composer has nothing left to show, so it
 * steps out of the way instead of sitting over the canvas.
 */
async function generate(input?: Parameters<typeof startGeneration>[0]) {
  if (!(await startGeneration(input))) return;
  projectLeave.updateDraft(false);
  emit("close");
}
async function retryInitialization() {
  initializationAttempted.value = false;
  await refresh();
}
defineExpose({ requestClose });
</script>

<template>
  <div class="flex min-h-0 flex-col" :aria-busy="loading || creatingProject">
    <SpeechNodeEditorHeader :node :project-type @close="emit('close')" />
    <div v-if="errors.length" class="space-y-1 bg-surface-danger-subtle px-3 py-2" role="alert">
      <p v-for="message in errors" :key="message" class="text-caption text-fg-danger">{{ message }}</p>
    </div>
    <div v-if="loading || creatingProject" class="flex items-center gap-2 p-3 text-caption text-fg-secondary">
      <UiSpinner size="sm" />{{ $t("common.loading") }}
    </div>
    <div v-if="!loading && !creatingProject && !selectedProject" class="space-y-2 p-3">
      <p v-if="!initialInput" class="text-caption text-fg-secondary" role="status">
        {{ $t("spaces.speech.project.sourceNotReady") }}
      </p>
      <UiButton v-else size="sm" @click="retryInitialization">{{ $t("common.retry") }}</UiButton>
    </div>
    <SpeechProjectEditor
      v-if="selectedProject"
      :key="`${selectedProject.project.id}:${draftEpoch}`"
      :source-node="node"
      :project="selectedProject"
      :voices="voices"
      :preview="preview"
      :preview-listening="previewListening"
      :output="publishedSpeechOutput(selectedProject.revision)"
      :progress="previewProgress"
      :previewing="previewing"
      :generating="generating"
      :saving="savingRevision"
      :submitting="generationSubmitting"
      :cancelling="previewCancelling"
      @draft-state="projectLeave.updateDraft"
      @preview="previewSelection"
      @generate="generate"
      @cancel="cancelPreview"
      @stop-listening="stopPreviewListening"
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

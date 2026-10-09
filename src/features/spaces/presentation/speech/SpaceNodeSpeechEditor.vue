<script setup lang="ts">
import { computed, shallowRef } from "vue";
import { RouterLink } from "vue-router";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { SpeechAudioOutput } from "../../contracts/generated/SpeechAudioOutput";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import type { NodeSpeechProjectType } from "../../speech/node-speech";
import { useSpeechProjectLeave } from "../../speech/use-speech-project-leave";
import { useSpeechWorkspace } from "../../speech/use-speech-workspace";
import SpaceUnsavedChangesDialog from "../workspace/SpaceUnsavedChangesDialog.vue";
import SpeechProjectEditor from "./SpeechProjectEditor.vue";
import SpeechProjectForm from "./SpeechProjectForm.vue";
import VoiceCandidateForm from "./VoiceCandidateForm.vue";
import VoiceProfileForm from "./VoiceProfileForm.vue";

const props = defineProps<{ node: SpaceCanvasNode; projectType: NodeSpeechProjectType }>();
const {
  voices,
  creatingVoice,
  addPresetVoice,
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

const store = useSpaceStore();
const addingVoice = shallowRef(false);
const draftEpoch = shallowRef(0);
const projectLeave = useSpeechProjectLeave(
  saveRevision,
  openProject,
  () => (draftEpoch.value += 1),
  settleRevisionSave,
);

const publishedOutput = computed<SpeechAudioOutput | undefined>(() => {
  const project = selectedProject.value;
  if (!project) return undefined;
  const revision = project.revision;
  if (
    !revision.outputAssetPath ||
    !revision.outputSha256 ||
    typeof revision.outputSampleRate !== "number" ||
    typeof revision.outputSampleCount !== "number"
  )
    return undefined;
  return {
    path: revision.outputAssetPath,
    sha256: revision.outputSha256,
    sampleRate: revision.outputSampleRate,
    sampleCount: revision.outputSampleCount,
  };
});
const projectOptions = computed(() =>
  projects.value.map((project) => ({ value: project.id, label: project.title })),
);
</script>

<template>
  <div class="h-full overflow-y-auto p-3">
    <div v-if="errors.length" class="mb-3 space-y-1 rounded-md bg-surface-danger-subtle p-2">
      <p v-for="message in errors" :key="message" class="text-caption text-fg-danger">{{ message }}</p>
    </div>
    <div class="space-y-5" :aria-busy="loading">
      <RouterLink
        :to="{ path: '/settings', query: { pane: 'voices' } }"
        class="text-caption text-fg-accent hover:underline"
        data-testid="manage-voices"
      >
        {{ $t("spaces.speech.library.manage") }}
      </RouterLink>
      <UiButton size="sm" variant="ghost" data-testid="add-reference-voice" @click="addingVoice = true">
        {{ $t("spaces.speech.library.add") }}
      </UiButton>
      <UiDialog v-if="addingVoice" v-model:open="addingVoice" :title="$t('spaces.speech.library.add')">
        <VoiceCandidateForm />
      </UiDialog>
      <section
        v-if="
          !loading &&
          !voices.some((voice) => voice.isActive && voice.archivedAt == null) &&
          store.activeSpaceId
        "
        class="space-y-2"
      >
        <p class="text-caption text-fg-secondary">{{ $t("spaces.speech.library.firstVoice") }}</p>
        <VoiceProfileForm :disabled="creatingVoice" @create="addPresetVoice" />
      </section>
      <section class="space-y-2 border-t border-stroke-divider pt-4">
        <h2 class="text-label font-medium text-fg-primary">{{ $t("spaces.speech.project.new") }}</h2>
        <fieldset>
          <SpeechProjectForm
            :node="node"
            :project-type="projectType"
            :disabled="creatingProject"
            @create="addProject"
          />
        </fieldset>
      </section>
      <UiSelect
        v-if="projects.length"
        :model-value="selectedProject?.project.id ?? ''"
        :options="projectOptions"
        :label="$t('spaces.speech.project.nodeProjects')"
        @update:model-value="projectLeave.selectProject"
      />
      <SpeechProjectEditor
        v-if="selectedProject"
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

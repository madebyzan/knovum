<script setup lang="ts">
import { computed } from "vue";

import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { SpeechAudioOutput } from "../../contracts/generated/SpeechAudioOutput";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpeechWorkspace } from "../../speech/use-speech-workspace";
import SpeechProjectEditor from "./SpeechProjectEditor.vue";
import SpeechProjectForm from "./SpeechProjectForm.vue";
import VoiceCandidateForm from "./VoiceCandidateForm.vue";
import VoiceManagement from "./VoiceManagement.vue";
import VoiceProfileForm from "./VoiceProfileForm.vue";

const props = defineProps<{ node: SpaceCanvasNode }>();
const {
  voices,
  projects,
  selectedProject,
  preview,
  previewListening,
  runState,
  generationProgress,
  activeJobId,
  creatingVoice,
  creatingProject,
  savingRevision,
  generationSubmitting,
  generationCancelling,
  loading,
  error,
  addPreset,
  addProject,
  acceptVoice,
  refreshVoices,
  openProject,
  saveRevision,
  previewSelection,
  generateProject,
  cancelGeneration,
  stopPreviewListening,
  exportRender,
} = useSpeechWorkspace(() => props.node.id);

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
    <p v-if="error" class="mb-3 rounded-md bg-surface-danger-subtle p-2 text-caption text-fg-danger">
      {{ error }}
    </p>
    <div class="space-y-5" :aria-busy="loading">
      <details class="space-y-2">
        <summary class="cursor-pointer text-label font-medium text-fg-primary">
          {{ $t("spaces.speech.voices") }}
        </summary>
        <fieldset>
          <VoiceProfileForm :disabled="creatingVoice" @create="addPreset" />
          <VoiceCandidateForm @created="acceptVoice" />
        </fieldset>
        <VoiceManagement @changed="refreshVoices" />
      </details>
      <section class="space-y-2 border-t border-stroke-divider pt-4">
        <h2 class="text-label font-medium text-fg-primary">{{ $t("spaces.speech.newProject") }}</h2>
        <fieldset>
          <SpeechProjectForm :node="node" :disabled="creatingProject" @create="addProject" />
        </fieldset>
      </section>
      <UiSelect
        v-if="projects.length"
        :model-value="selectedProject?.project.id ?? ''"
        :options="projectOptions"
        :label="$t('spaces.speech.nodeProjects')"
        @update:model-value="openProject"
      />
      <SpeechProjectEditor
        v-if="selectedProject"
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
        @preview="previewSelection"
        @generate="generateProject"
        @cancel="cancelGeneration"
        @stop-listening="stopPreviewListening"
        @export="exportRender"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";

import type { SpeechAudioOutput } from "./contracts";
import SpeechCatalog from "./SpeechCatalog.vue";
import SpeechProjectEditor from "./SpeechProjectEditor.vue";
import SpeechProjectForm from "./SpeechProjectForm.vue";
import { useSpeechWorkspace } from "./use-speech-workspace";
import { activeVoiceRevisions } from "./voice-options";
import VoiceCandidateForm from "./VoiceCandidateForm.vue";
import VoiceManagement from "./VoiceManagement.vue";
import VoiceProfileForm from "./VoiceProfileForm.vue";

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
} = useSpeechWorkspace();

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
const catalogVoices = computed(() => activeVoiceRevisions(voices.value));
</script>

<template>
  <div class="h-full overflow-y-auto p-3">
    <p v-if="error" class="mb-3 rounded-md bg-surface-danger-subtle p-2 text-caption text-fg-danger">
      {{ error }}
    </p>
    <div class="space-y-5" :aria-busy="loading">
      <section class="space-y-2">
        <h2 class="text-label font-medium text-fg-primary">{{ $t("spaces.speech.newVoice") }}</h2>
        <fieldset>
          <VoiceProfileForm :disabled="creatingVoice" @create="addPreset" />
          <VoiceCandidateForm @created="acceptVoice" />
        </fieldset>
      </section>
      <VoiceManagement @changed="refreshVoices" />
      <section class="space-y-2 border-t border-stroke-divider pt-4">
        <h2 class="text-label font-medium text-fg-primary">{{ $t("spaces.speech.newProject") }}</h2>
        <fieldset>
          <SpeechProjectForm :disabled="creatingProject" @create="addProject" />
        </fieldset>
      </section>
      <SpeechProjectEditor
        v-if="selectedProject"
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
      <SpeechCatalog :voices="catalogVoices" :projects="projects" @select-project="openProject" />
    </div>
  </div>
</template>
